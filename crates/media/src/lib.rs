use anyhow::{ensure, Result};
use cctv_core::{Camera, Credentials, StreamKind, Transport};
use std::{
    collections::HashMap,
    ffi::{c_char, c_int, c_void, CString},
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
        Arc, Mutex, Once, Weak,
    },
    thread,
    time::{Duration, Instant},
};

unsafe extern "C" {
    fn cctv_media_init();
    fn cctv_decode(
        url: *const c_char,
        tcp: c_int,
        timeout_ms: c_int,
        hardware: c_int,
        input_kind: c_int,
        user: *mut c_void,
        control: extern "C" fn(*mut c_void, c_int) -> c_int,
        video: extern "C" fn(*mut c_void, *const u8, c_int, c_int, c_int, i64),
    ) -> c_int;
}
#[derive(Clone)]
pub struct Frame {
    pub pixels: Vec<u8>,
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub sequence: u64,
    pub received: Instant,
}
#[derive(Clone, Debug)]
pub enum Status {
    Connecting,
    Playing,
    Reconnecting { attempt: u32, code: i32 },
    Stopped,
    Error(String),
}
pub struct Session {
    pub camera_id: String,
    frame: Mutex<Option<Arc<Frame>>>,
    status: Mutex<Status>,
    cancel: AtomicBool,
    audio: Arc<AudioPolicy>,
    frames: AtomicU64,
    replaced: AtomicU64,
    consumed: AtomicU64,
    hardware: AtomicBool,
    audio_bytes: AtomicU64,
}
impl Session {
    pub fn latest(&self) -> Option<Arc<Frame>> {
        let frame = self.frame.lock().ok().and_then(|v| v.clone());
        if let Some(f) = &frame {
            self.consumed.store(f.sequence, Ordering::Relaxed);
        }
        frame
    }
    pub fn status(&self) -> Status {
        self.status
            .lock()
            .map(|v| v.clone())
            .unwrap_or(Status::Stopped)
    }
    pub fn metrics(&self) -> (u64, u64) {
        (
            self.frames.load(Ordering::Relaxed),
            self.replaced.load(Ordering::Relaxed),
        )
    }
    fn set_status(&self, v: Status) {
        if let Ok(mut s) = self.status.lock() {
            *s = v;
        }
    }
    pub fn stop(&self) {
        self.cancel.store(true, Ordering::Release);
        self.set_status(Status::Stopped);
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
    }
}
#[derive(Default)]
pub struct AudioPolicy {
    active: Mutex<Option<String>>,
    volume: AtomicU32,
}
impl AudioPolicy {
    pub fn select(&self, id: Option<String>) {
        if let Ok(mut a) = self.active.lock() {
            *a = id;
        }
    }
    pub fn set_volume(&self, v: f64) {
        self.volume
            .store((v.clamp(0., 1.) * 100.) as u32, Ordering::Relaxed);
    }
    pub fn is_active(&self, id: &str) -> bool {
        self.active.lock().is_ok_and(|a| a.as_deref() == Some(id))
    }
}
#[derive(Default)]
pub struct SessionManager {
    sessions: HashMap<String, Weak<Session>>,
    pub audio: Arc<AudioPolicy>,
}
impl SessionManager {
    pub fn acquire(
        &mut self,
        camera: &Camera,
        credentials: Credentials,
        focus: bool,
    ) -> Result<Arc<Session>> {
        camera.validate()?;
        static INIT: Once = Once::new();
        // SAFETY: one-time initialization before decoding; libav logging is muted to avoid URL leaks.
        INIT.call_once(|| unsafe { cctv_media_init() });
        let source = if focus {
            &camera.stream_url
        } else {
            camera.substream_url.as_ref().unwrap_or(&camera.stream_url)
        };
        let key = format!(
            "{}:{source}:{:?}:{}:{}:{:?}:{}:{}",
            camera.id,
            camera.transport,
            camera.hardware_decode,
            camera.timeout_ms,
            camera.stream_kind,
            camera.mjpeg_multipart,
            camera.snapshot_interval_ms
        );
        self.sessions.retain(|_, v| v.strong_count() > 0);
        if let Some(s) = self.sessions.get(&key).and_then(Weak::upgrade) {
            return Ok(s);
        }
        let mut url = url::Url::parse(source)?;
        if !credentials.username.is_empty() {
            url.set_username(&credentials.username)
                .map_err(|_| anyhow::anyhow!("Usuário inválido"))?;
            url.set_password(Some(&credentials.password))
                .map_err(|_| anyhow::anyhow!("Credencial inválida"))?;
        }
        let source = CString::new(url.as_str())?;
        let session = Arc::new(Session {
            camera_id: camera.id.clone(),
            frame: Mutex::new(None),
            status: Mutex::new(Status::Connecting),
            cancel: AtomicBool::new(false),
            audio: self.audio.clone(),
            frames: AtomicU64::new(0),
            replaced: AtomicU64::new(0),
            consumed: AtomicU64::new(0),
            hardware: AtomicBool::new(false),
            audio_bytes: AtomicU64::new(0),
        });
        self.sessions.insert(key, Arc::downgrade(&session));
        let weak = Arc::downgrade(&session);
        let tcp = matches!(camera.transport, Transport::Tcp);
        let timeout = camera.timeout_ms;
        let snapshot = camera.stream_kind == StreamKind::Jpeg;
        let hardware = camera.hardware_decode && !snapshot;
        let snapshot_interval = camera.snapshot_interval_ms;
        let input_kind = match camera.stream_kind {
            StreamKind::Mjpeg if camera.mjpeg_multipart => 1,
            StreamKind::Mjpeg => 2,
            StreamKind::Jpeg => 3,
            _ => 0,
        };
        // Worker owns only a Weak, so last consumer dropping immediately cancels I/O via callback.
        thread::Builder::new()
            .name("cctv-decode".into())
            .spawn(move || {
                let mut callback = Callback {
                    session: weak.clone(),
                };
                let mut attempt = 0;
                loop {
                    let Some(s) = weak.upgrade() else { break };
                    if s.cancel.load(Ordering::Acquire) {
                        break;
                    }
                    if !snapshot || s.latest().is_none() {
                        s.set_status(Status::Connecting);
                    }
                    drop(s);
                    // SAFETY: callback and C string live until synchronous decoder returns; callbacks copy pixels.
                    let mut code = unsafe {
                        cctv_decode(
                            source.as_ptr(),
                            tcp as c_int,
                            timeout as c_int,
                            hardware as c_int,
                            input_kind,
                            &mut callback as *mut _ as *mut c_void,
                            control,
                            video,
                        )
                    };
                    if hardware
                        && code < 0
                        && weak
                            .upgrade()
                            .is_some_and(|s| !s.cancel.load(Ordering::Acquire))
                    {
                        // SAFETY: identical lifetime contract; retry software after device/decoder failure.
                        code = unsafe {
                            cctv_decode(
                                source.as_ptr(),
                                tcp as c_int,
                                timeout as c_int,
                                0,
                                input_kind,
                                &mut callback as *mut _ as *mut c_void,
                                control,
                                video,
                            )
                        };
                    }
                    let Some(s) = weak.upgrade() else { break };
                    if s.cancel.load(Ordering::Acquire) {
                        s.set_status(Status::Stopped);
                        break;
                    }
                    let wait_ms = if snapshot && code == 0 {
                        attempt = 0;
                        u64::from(snapshot_interval)
                    } else {
                        attempt += 1;
                        s.set_status(Status::Reconnecting { attempt, code });
                        1000 * (1u64 << attempt.min(4))
                    };
                    drop(s);
                    let deadline = Instant::now() + Duration::from_millis(wait_ms);
                    while Instant::now() < deadline {
                        if weak
                            .upgrade()
                            .is_none_or(|s| s.cancel.load(Ordering::Acquire))
                        {
                            return;
                        }
                        thread::sleep(Duration::from_millis(25));
                    }
                }
            })?;
        Ok(session)
    }
    pub fn statistics(&self) -> Vec<Statistics> {
        self.sessions
            .values()
            .filter_map(Weak::upgrade)
            .map(|s| Statistics {
                camera_id: s.camera_id.clone(),
                frames: s.frames.load(Ordering::Relaxed),
                dropped: s.replaced.load(Ordering::Relaxed),
                hardware: s.hardware.load(Ordering::Relaxed),
                audio_bytes: s.audio_bytes.load(Ordering::Relaxed),
                frame_age_ms: s
                    .frame
                    .lock()
                    .ok()
                    .and_then(|f| f.as_ref().map(|f| f.received.elapsed().as_millis())),
            })
            .collect()
    }
    pub fn invalidate(&mut self, camera_id: &str) {
        self.sessions.retain(|_, weak| {
            if let Some(s) = weak.upgrade() {
                if s.camera_id == camera_id {
                    s.stop();
                    return false;
                }
            }
            true
        });
    }
    pub fn shutdown(&mut self) {
        for s in self.sessions.values().filter_map(Weak::upgrade) {
            s.stop();
        }
        self.sessions.clear();
        self.audio.select(None);
    }
}
impl Drop for SessionManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}
struct Callback {
    session: Weak<Session>,
}
extern "C" fn control(user: *mut c_void, mode: c_int) -> c_int {
    // SAFETY: decoder passes the same exclusive Callback pointer valid for the entire call.
    let callback = unsafe { &*(user as *const Callback) };
    let Some(s) = callback.session.upgrade() else {
        return if mode == 0 { 1 } else { 0 };
    };
    if mode == 2 {
        s.hardware.store(true, Ordering::Relaxed);
        return 0;
    }
    if mode == 3 {
        s.hardware.store(false, Ordering::Relaxed);
        return 0;
    }
    if mode > 100 {
        s.audio_bytes
            .fetch_add(mode as u64 - 100, Ordering::Relaxed);
        return 0;
    }
    if mode == 0 {
        i32::from(s.cancel.load(Ordering::Acquire))
    } else if s.audio.is_active(&s.camera_id) {
        s.audio.volume.load(Ordering::Relaxed) as i32
    } else {
        0
    }
}
extern "C" fn video(
    user: *mut c_void,
    pixels: *const u8,
    width: c_int,
    height: c_int,
    stride: c_int,
    _pts: i64,
) {
    // SAFETY: decoder guarantees Callback pointer and stride*height BGRA bytes valid until return.
    let callback = unsafe { &*(user as *const Callback) };
    if let Some(s) = callback.session.upgrade() {
        if s.cancel.load(Ordering::Acquire) {
            return;
        }
        if width <= 0 || height <= 0 || stride < width * 4 || height > 8192 {
            return;
        }
        // SAFETY: dimensions checked in C and above; copying severs lifetime from decoder buffer.
        let pixels =
            unsafe { std::slice::from_raw_parts(pixels, (stride as usize) * (height as usize)) }
                .to_vec();
        let sequence = s.frames.fetch_add(1, Ordering::Relaxed) + 1;
        if let Ok(mut slot) = s.frame.lock() {
            if slot
                .as_ref()
                .is_some_and(|f| f.sequence > s.consumed.load(Ordering::Relaxed))
            {
                s.replaced.fetch_add(1, Ordering::Relaxed);
            }
            *slot = Some(Arc::new(Frame {
                pixels,
                width,
                height,
                stride,
                sequence,
                received: Instant::now(),
            }));
        }
        s.set_status(Status::Playing);
    }
}
pub fn connection_test(camera: &Camera, credentials: Credentials) -> Result<()> {
    let mut m = SessionManager::default();
    let s = m.acquire(camera, credentials, true)?;
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(u64::from(camera.timeout_ms) * 3) {
        if s.latest().is_some() {
            m.shutdown();
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
    m.shutdown();
    ensure!(
        s.latest().is_some(),
        "Nenhum frame recebido. Verifique endereço, credenciais, codec e rede."
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn audio_is_exclusive() {
        let a = AudioPolicy::default();
        a.select(Some("one".into()));
        assert!(a.is_active("one"));
        a.select(Some("two".into()));
        assert!(!a.is_active("one"));
        a.set_volume(2.);
        assert_eq!(a.volume.load(Ordering::Relaxed), 100);
    }
    #[test]
    fn shares_and_releases_sessions() -> Result<()> {
        let c = Camera {
            name: "offline".into(),
            stream_url: "rtsp://127.0.0.1:1/live".into(),
            timeout_ms: 500,
            hardware_decode: false,
            ..Default::default()
        };
        let mut manager = SessionManager::default();
        let a = manager.acquire(&c, Credentials::default(), false)?;
        let b = manager.acquire(&c, Credentials::default(), false)?;
        assert!(Arc::ptr_eq(&a, &b));
        let weak = Arc::downgrade(&a);
        drop(a);
        drop(b);
        thread::sleep(Duration::from_millis(50));
        assert!(weak.upgrade().is_none());
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Statistics {
    pub camera_id: String,
    pub frames: u64,
    pub dropped: u64,
    pub hardware: bool,
    pub audio_bytes: u64,
    pub frame_age_ms: Option<u128>,
}
