use cctv_core::{Camera, Credentials};
use cctv_media::{SessionManager, Status};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
const VIDEO: &[u8] = include_bytes!("fixtures/sample.ts");
struct Server {
    url: String,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Server {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture server");
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}/live.ts", listener.local_addr().unwrap());
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let worker = thread::spawn(move || {
            while !flag.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        thread::spawn(move || {
                            stream
                                .set_read_timeout(Some(Duration::from_secs(2)))
                                .unwrap();
                            let mut buf = [0; 2048];
                            let _ = stream.read(&mut buf);
                            let header=format!("HTTP/1.1 200 OK\r\nContent-Type: video/mp2t\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",VIDEO.len());
                            let _ = stream.write_all(header.as_bytes());
                            let _ = stream.write_all(VIDEO);
                        });
                    }
                    Err(_) => thread::sleep(Duration::from_millis(10)),
                }
            }
        });
        Self {
            url,
            stop,
            worker: Some(worker),
        }
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(w) = self.worker.take() {
            w.join().unwrap();
        }
    }
}
#[test]
fn real_h264_aac_decode_four_streams_and_reconnect() {
    let server = Server::start();
    let mut manager = SessionManager::default();
    let cameras: Vec<_> = (0..4)
        .map(|i| Camera {
            name: format!("Camera {i}"),
            stream_url: server.url.clone(),
            hardware_decode: false,
            timeout_ms: 1000,
            ..Default::default()
        })
        .collect();
    let sessions: Vec<_> = cameras
        .iter()
        .map(|c| manager.acquire(c, Credentials::default(), false).unwrap())
        .collect();
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(8) && sessions.iter().any(|s| s.latest().is_none())
    {
        thread::sleep(Duration::from_millis(25));
    }
    for s in &sessions {
        let frame = s.latest().expect("real BGRA frame");
        assert_eq!((frame.width, frame.height), (320, 180));
        assert_eq!(
            frame.pixels.len(),
            frame.stride as usize * frame.height as usize
        );
        assert!(frame.pixels.iter().any(|p| *p != 0));
    }
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(8) && sessions[0].metrics().0 < 25 {
        thread::sleep(Duration::from_millis(25));
    }
    assert!(
        sessions[0].metrics().0 >= 25,
        "stream must reconnect after EOF"
    );
    assert!(
        sessions[0].metrics().1 > 0,
        "obsolete frames must be discarded"
    );
    manager.shutdown();
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(3)
        && sessions
            .iter()
            .any(|s| !matches!(s.status(), Status::Stopped))
    {
        thread::sleep(Duration::from_millis(25));
    }
    assert!(sessions
        .iter()
        .all(|s| matches!(s.status(), Status::Stopped)));
}
#[test]
fn jpeg_is_decoded_by_native_library() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/snapshot.jpg", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0; 2048];
        let _ = stream.read(&mut request);
        let jpeg = include_bytes!("fixtures/sample.jpg");
        write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",jpeg.len()).unwrap();
        stream.write_all(jpeg).unwrap();
    });
    let c = Camera {
        name: "JPEG".into(),
        stream_url: url,
        hardware_decode: false,
        ..Default::default()
    };
    assert!(cctv_media::connection_test(&c, Credentials::default()).is_ok());
    server.join().unwrap();
}

#[test]
#[ignore = "requires a private PulseAudio null sink; see docs/TESTING.md"]
fn real_pcm_output_and_exclusive_mute() {
    assert!(
        std::env::var("PULSE_SERVER").is_ok(),
        "Use a private null sink, never the user's speakers"
    );
    let server = Server::start();
    let mut manager = SessionManager::default();
    let a = Camera {
        name: "Audio A".into(),
        stream_url: server.url.clone(),
        hardware_decode: false,
        ..Default::default()
    };
    let b = Camera {
        name: "Audio B".into(),
        stream_url: server.url.clone(),
        hardware_decode: false,
        ..Default::default()
    };
    let _sa = manager.acquire(&a, Credentials::default(), false).unwrap();
    let _sb = manager.acquire(&b, Credentials::default(), false).unwrap();
    manager.audio.set_volume(0.3);
    manager.audio.select(Some(a.id.clone()));
    let bytes = |id: &str| {
        manager
            .statistics()
            .iter()
            .find(|s| s.camera_id == id)
            .map(|s| s.audio_bytes)
            .unwrap_or(0)
    };
    let start = Instant::now();
    while bytes(&a.id) == 0 && start.elapsed() < Duration::from_secs(6) {
        thread::sleep(Duration::from_millis(25));
    }
    assert!(bytes(&a.id) > 0, "PCM must reach PulseAudio");
    assert_eq!(bytes(&b.id), 0);
    manager.audio.select(Some(b.id.clone()));
    thread::sleep(Duration::from_millis(500));
    let muted = bytes(&a.id);
    let start = Instant::now();
    while bytes(&b.id) == 0 && start.elapsed() < Duration::from_secs(6) {
        thread::sleep(Duration::from_millis(25));
    }
    assert!(bytes(&b.id) > 0);
    assert_eq!(bytes(&a.id), muted, "previous camera stays muted");
    manager.shutdown();
}
