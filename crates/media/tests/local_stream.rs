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

#[test]
fn explicit_mjpeg_formats_and_snapshot_refresh() {
    use cctv_core::StreamKind;
    use std::sync::Mutex;
    for (kind, multipart) in [
        (StreamKind::Jpeg, true),
        (StreamKind::Mjpeg, true),
        (StreamKind::Mjpeg, false),
    ] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let endpoint = format!("http://{}/image", listener.local_addr().unwrap());
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let received = requests.clone();
        let worker = thread::spawn(move || {
            let jpeg = include_bytes!("fixtures/sample.jpg");
            while !flag.load(Ordering::Acquire) {
                if let Ok((mut stream, _)) = listener.accept() {
                    stream
                        .set_read_timeout(Some(Duration::from_secs(1)))
                        .unwrap();
                    let mut buf = [0; 2048];
                    let _ = stream.read(&mut buf);
                    received.lock().unwrap().push(Instant::now());
                    let mut body = vec![];
                    let mime = if kind == StreamKind::Jpeg {
                        "image/jpeg"
                    } else if multipart {
                        "multipart/x-mixed-replace; boundary=frame"
                    } else {
                        "video/x-motion-jpeg"
                    };
                    if kind == StreamKind::Jpeg {
                        body.extend_from_slice(jpeg);
                    } else {
                        for _ in 0..16 {
                            if multipart {
                                write!(body, "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n", jpeg.len()).unwrap();
                            }
                            body.extend_from_slice(jpeg);
                            if multipart {
                                body.extend_from_slice(b"\r\n");
                            }
                        }
                        if multipart {
                            body.extend_from_slice(b"--frame--\r\n");
                        }
                    }
                    let _ = write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
                    let _ = stream.write_all(&body);
                } else {
                    thread::sleep(Duration::from_millis(5));
                }
            }
        });
        let camera = Camera {
            name: "Explicit image format".into(),
            stream_url: endpoint,
            stream_kind: kind,
            mjpeg_multipart: multipart,
            snapshot_interval_ms: 200,
            hardware_decode: false,
            timeout_ms: 1000,
            ..Default::default()
        };
        let mut manager = SessionManager::default();
        let session = manager
            .acquire(&camera, Credentials::default(), true)
            .unwrap();
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(5) && session.metrics().0 < 3 {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(
            session.metrics().0 >= 3,
            "{kind:?} multipart={multipart} must decode multiple real frames"
        );
        assert_eq!(session.latest().unwrap().width, 320);
        if kind == StreamKind::Jpeg {
            let times = requests.lock().unwrap();
            assert!(times[2].duration_since(times[0]) >= Duration::from_millis(380));
            assert!(
                times[2].duration_since(times[0]) < Duration::from_secs(2),
                "snapshot must use its interval, not reconnection backoff"
            );
            assert!(matches!(session.status(), Status::Playing));
        }
        manager.shutdown();
        thread::sleep(Duration::from_millis(80));
        let ended = requests.lock().unwrap().len();
        thread::sleep(Duration::from_millis(300));
        assert_eq!(
            requests.lock().unwrap().len(),
            ended,
            "shutdown cancels refresh/reconnect"
        );
        stop.store(true, Ordering::Release);
        worker.join().unwrap();
    }
}

#[test]
fn anonymous_mode_never_answers_auth_challenge() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let endpoint = format!("http://{}/snapshot.jpg", listener.local_addr().unwrap());
    let stop = Arc::new(AtomicBool::new(false));
    let flag = stop.clone();
    let authenticated = Arc::new(AtomicBool::new(false));
    let seen = authenticated.clone();
    let worker = thread::spawn(move || {
        while !flag.load(Ordering::Acquire) {
            if let Ok((mut stream, _)) = listener.accept() {
                stream
                    .set_read_timeout(Some(Duration::from_secs(1)))
                    .unwrap();
                let mut bytes = vec![];
                let mut buf = [0; 2048];
                while bytes.len() < 8192 && !bytes.ends_with(b"\r\n\r\n") {
                    let n = stream.read(&mut buf).unwrap_or(0);
                    if n == 0 {
                        break;
                    }
                    bytes.extend_from_slice(&buf[..n]);
                }
                if String::from_utf8_lossy(&bytes)
                    .to_ascii_lowercase()
                    .contains("\r\nauthorization: basic ")
                {
                    seen.store(true, Ordering::Release);
                    let jpeg = include_bytes!("fixtures/sample.jpg");
                    let _ = write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", jpeg.len());
                    let _ = stream.write_all(jpeg);
                } else {
                    let _ = stream.write_all(b"HTTP/1.1 401 Unauthorized\r\nWWW-Authenticate: Basic realm=\"fixture\"\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
                }
            } else {
                thread::sleep(Duration::from_millis(5));
            }
        }
    });
    let mut camera = Camera {
        name: "Authentication challenge".into(),
        stream_url: endpoint,
        no_credentials: true,
        timeout_ms: 500,
        hardware_decode: false,
        ..Default::default()
    };
    let credentials = || Credentials {
        username: "fixture".into(),
        password: "fixture-only".into(),
    };
    let anonymous_result = cctv_media::connection_test(&camera, credentials());
    let anonymous_sent_auth = authenticated.load(Ordering::Acquire);
    camera.no_credentials = false;
    let authenticated_result = cctv_media::connection_test(&camera, credentials());
    stop.store(true, Ordering::Release);
    worker.join().unwrap();
    assert!(
        anonymous_result.is_err(),
        "Anonymous mode must not authenticate even when credentials were passed"
    );
    assert!(
        !anonymous_sent_auth,
        "A challenge must never disclose credentials in anonymous mode"
    );
    assert!(
        authenticated_result.is_ok(),
        "Control fixture must accept the same credentials when enabled"
    );
    assert!(authenticated.load(Ordering::Acquire));
}
