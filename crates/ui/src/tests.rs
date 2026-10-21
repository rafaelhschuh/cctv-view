use super::*;
fn widgets(widget: &impl IsA<gtk::Widget>) -> Vec<gtk::Widget> {
    let mut all = vec![];
    let mut child = widget.first_child();
    while let Some(w) = child {
        child = w.next_sibling();
        all.extend(widgets(&w));
        all.push(w);
    }
    all
}
fn pump_until(test: impl Fn() -> bool) {
    let start = std::time::Instant::now();
    while !test() && start.elapsed() < Duration::from_secs(8) {
        let c = glib::MainContext::default();
        while c.pending() {
            c.iteration(false);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(test(), "GTK action timed out");
}
fn dialog(title: &str) -> gtk::Dialog {
    gtk::Window::list_toplevels()
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Dialog>().ok())
        .find(|w| w.title().as_deref() == Some(title))
        .expect("dialog should be visible")
}
#[test]
#[ignore = "requires GTK display; run under xvfb-run as documented"]
fn native_crud_editor_and_focus() {
    gtk::init().unwrap();
    let app = gtk::Application::builder()
        .application_id("br.dev.schuh.cctv-view.tests")
        .build();
    app.register(None::<&gio::Cancellable>).unwrap();
    let directory = std::env::temp_dir().join(cctv_core::id());
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("test.db");
    Store::open(&path).unwrap();
    let w = gtk::ApplicationWindow::builder().application(&app).build();
    let s = build(
        w,
        path.clone(),
        vec![],
        vec![],
        Preferences::default(),
        Arc::new(tokio::runtime::Runtime::new().unwrap()),
    );
    s.window.present();
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let stream_url = format!("http://{}/live.ts", listener.local_addr().unwrap());
    listener.set_nonblocking(true).unwrap();
    let server_stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = server_stop.clone();
    let server = std::thread::spawn(move || {
        while !flag.load(std::sync::atomic::Ordering::Acquire) {
            if let Ok((mut stream, _)) = listener.accept() {
                std::thread::spawn(move || {
                    stream
                        .set_read_timeout(Some(Duration::from_secs(1)))
                        .unwrap();
                    let mut buf = [0; 2048];
                    let _ = stream.read(&mut buf);
                    let bytes = include_bytes!("../../media/tests/fixtures/sample.ts");
                    let _=write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: video/mp2t\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",bytes.len());
                    let _ = stream.write_all(bytes);
                });
            } else {
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    });
    camera::form(&s, Camera::default(), None);
    let d = dialog("Configurar câmera");
    let all = widgets(&d);
    for (tooltip, value) in [
        ("Nome", "Portaria de teste"),
        ("URL do stream", stream_url.as_str()),
    ] {
        all.iter()
            .filter_map(|w| w.downcast_ref::<gtk::Entry>())
            .find(|e| e.tooltip_text().as_deref() == Some(tooltip))
            .unwrap()
            .set_text(value);
    }
    d.response(gtk::ResponseType::Accept);
    pump_until(|| s.cameras.borrow().len() == 1);
    assert_eq!(
        Store::open(&path).unwrap().cameras().unwrap()[0].name,
        "Portaria de teste"
    );
    editor::open(&s, View::default());
    let d = dialog("Editor de Views");
    let all = widgets(&d);
    let name = all
        .iter()
        .filter_map(|w| w.downcast_ref::<gtk::Entry>())
        .find(|e| e.placeholder_text().as_deref() == Some("Nome da View"))
        .unwrap();
    name.set_text("Entrada");
    let preset = all
        .iter()
        .filter_map(|w| w.downcast_ref::<gtk::DropDown>())
        .next()
        .unwrap();
    preset.set_selected(3);
    d.response(gtk::ResponseType::Accept);
    pump_until(|| s.views.borrow().len() == 1);
    let v = s.views.borrow()[0].clone();
    assert_eq!(v.panels.len(), 1);
    assert_eq!(v.name, "Entrada");
    assert_eq!(Store::open(&path).unwrap().views().unwrap()[0], v);
    let tile = s.grid.child_at(0, 0).unwrap();
    let controllers = tile.observe_controllers();
    let click = (0..controllers.n_items())
        .filter_map(|i| controllers.item(i)?.downcast::<gtk::GestureClick>().ok())
        .find(|gesture| gesture.button() == 1)
        .unwrap();
    click.emit_by_name::<()>("pressed", &[&2i32, &0f64, &0f64]);
    assert_eq!(*s.focus.borrow(), Some(v.panels[0].id.clone()));
    assert!(s.grid.first_child().is_some());
    let controllers = s.window.observe_controllers();
    let keys = (0..controllers.n_items())
        .filter_map(|i| {
            controllers
                .item(i)?
                .downcast::<gtk::EventControllerKey>()
                .ok()
        })
        .next()
        .unwrap();
    use glib::translate::IntoGlib;
    assert!(keys.emit_by_name::<bool>(
        "key-pressed",
        &[
            &gdk::Key::Escape.into_glib(),
            &0u32,
            &gdk::ModifierType::empty()
        ]
    ));
    assert!(s.focus.borrow().is_none());
    assert_eq!(s.views.borrow()[0], v);
    let camera = s.cameras.borrow()[0].clone();
    let session = s
        .manager
        .lock()
        .unwrap()
        .acquire(&camera, Credentials::default(), false)
        .unwrap();
    pump_until(|| session.latest().is_some());
    let until = std::time::Instant::now() + Duration::from_millis(200);
    pump_until(|| std::time::Instant::now() >= until);
    if let Some(output) = std::env::var_os("CCTV_TEST_SCREENSHOT") {
        let paintable = gtk::WidgetPaintable::new(Some(&s.window));
        let snapshot = gtk::Snapshot::new();
        paintable.snapshot(&snapshot, s.window.width() as f64, s.window.height() as f64);
        let node = snapshot.to_node().unwrap();
        let texture = s.window.renderer().unwrap().render_texture(&node, None);
        texture.save_to_png(PathBuf::from(output)).unwrap();
    }
    // Deliver the actual GTK drop signal and verify the integrated draft redraw.
    editor::open(&s, v.clone());
    let draft_dialog = dialog("Editor de Views");
    let empty = widgets(&draft_dialog)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Button>().ok())
        .find(|b| b.label().as_deref() == Some("+"))
        .unwrap();
    let controllers = empty.observe_controllers();
    let drop_target = (0..controllers.n_items())
        .filter_map(|i| controllers.item(i)?.downcast::<gtk::DropTarget>().ok())
        .next()
        .unwrap();
    let value = glib::BoxedValue(format!("camera:{}", camera.id).to_value());
    assert!(drop_target.emit_by_name::<bool>("drop", &[&value, &0f64, &0f64]));
    assert_eq!(
        widgets(&draft_dialog)
            .into_iter()
            .filter_map(|w| w.downcast::<gtk::Button>().ok())
            .filter(|b| b
                .label()
                .is_some_and(|l| l.starts_with("Portaria de teste") && l.contains('\n')))
            .count(),
        2
    );
    draft_dialog.response(gtk::ResponseType::Cancel);
    // Numeric resizing exercises the same validated draft used by the drag handle.
    editor::open(&s, v.clone());
    let draft_dialog = dialog("Editor de Views");
    let panel_button = widgets(&draft_dialog)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Button>().ok())
        .find(|b| {
            b.label()
                .is_some_and(|l| l.starts_with("Portaria de teste") && l.contains('\n'))
        })
        .unwrap();
    panel_button.emit_clicked();
    let panel_dialog = dialog("Configurar painel");
    let controls: Vec<_> = widgets(&panel_dialog)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::SpinButton>().ok())
        .collect();
    assert_eq!(controls.len(), 4);
    controls[2].set_value(2.);
    controls[3].set_value(2.);
    panel_dialog.response(gtk::ResponseType::Accept);
    assert!(widgets(&draft_dialog)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Button>().ok())
        .any(|b| b.label().is_some_and(|l| l.contains("2 × 2"))));
    // Cancel edits must preserve persistent state.
    draft_dialog.response(gtk::ResponseType::Cancel);
    assert_eq!(Store::open(&path).unwrap().views().unwrap()[0], v);
    s.window.close();
    jobs::flush_preferences();
    std::fs::remove_dir_all(directory).unwrap();
    server_stop.store(true, std::sync::atomic::Ordering::Release);
    server.join().unwrap();
    let directory = std::env::temp_dir().join(cctv_core::id());
    std::fs::create_dir_all(&directory).unwrap();
    let state2 = build(
        gtk::ApplicationWindow::builder().application(&app).build(),
        directory.join("test.db"),
        vec![],
        vec![],
        Preferences::default(),
        Arc::new(tokio::runtime::Runtime::new().unwrap()),
    );
    state2.window.present();
    let weak_state = Rc::downgrade(&state2);
    state2.window.close();
    drop(state2);
    pump_until(|| weak_state.upgrade().is_none());
    jobs::flush_preferences();
    std::fs::remove_dir_all(directory).unwrap();
}
