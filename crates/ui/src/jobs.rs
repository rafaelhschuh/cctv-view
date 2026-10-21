use gtk::glib;
use std::{sync::mpsc, time::Duration};
/// Delivers a worker result on the main context. No GTK objects cross thread boundaries.
pub fn run<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
    done: impl FnOnce(T) + 'static,
) {
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let value = work();
        let _ = sender.send(value);
    });
    let mut done = Some(done);
    glib::timeout_add_local(Duration::from_millis(30), move || {
        match receiver.try_recv() {
            Ok(value) => {
                if let Some(done) = done.take() {
                    done(value);
                }
                glib::ControlFlow::Break
            }
            Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(_) => glib::ControlFlow::Break,
        }
    });
}

type Work = Box<dyn FnOnce() + Send>;
fn preferences_queue() -> &'static mpsc::Sender<Work> {
    static QUEUE: std::sync::OnceLock<mpsc::Sender<Work>> = std::sync::OnceLock::new();
    QUEUE.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<Work>();
        std::thread::spawn(move || {
            for work in rx {
                work();
            }
        });
        tx
    })
}
/// Preference snapshots are committed in main-context order, including during rapid volume changes.
pub fn preferences<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
    done: impl FnOnce(T) + 'static,
) {
    let (sender, receiver) = mpsc::sync_channel(1);
    let _ = preferences_queue().send(Box::new(move || {
        let _ = sender.send(work());
    }));
    let mut done = Some(done);
    glib::timeout_add_local(Duration::from_millis(30), move || {
        match receiver.try_recv() {
            Ok(value) => {
                if let Some(done) = done.take() {
                    done(value);
                }
                glib::ControlFlow::Break
            }
            Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(_) => glib::ControlFlow::Break,
        }
    });
}
pub fn flush_preferences() {
    let (tx, rx) = mpsc::sync_channel(1);
    let _ = preferences_queue().send(Box::new(move || {
        let _ = tx.send(());
    }));
    let _ = rx.recv_timeout(Duration::from_secs(4));
}
