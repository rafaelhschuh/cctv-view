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
fn button_named(root: &impl IsA<gtk::Widget>, text: &str) -> gtk::Button {
    widgets(root)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Button>().ok())
        .find(|b| b.label().as_deref() == Some(text))
        .unwrap_or_else(|| panic!("Button not found: {text}"))
}
fn entry_named(root: &impl IsA<gtk::Widget>, text: &str) -> gtk::Entry {
    widgets(root)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Entry>().ok())
        .find(|e| e.tooltip_text().as_deref() == Some(text))
        .unwrap_or_else(|| panic!("Entry not found: {text}"))
}
fn settle() {
    let until = std::time::Instant::now() + Duration::from_millis(200);
    pump_until(|| std::time::Instant::now() >= until);
}
fn capture(root: &impl IsA<gtk::Window>, name: &str) {
    let Some(folder) = std::env::var_os("CCTV_UI_SCREENSHOTS") else {
        return;
    };
    settle();
    let root = root.as_ref();
    std::fs::create_dir_all(&folder).unwrap();
    let paintable = gtk::WidgetPaintable::new(Some(root));
    // A freshly created WidgetPaintable may have no render node until the next GTK frame.
    let node = RefCell::new(None);
    root.queue_draw();
    pump_until(|| {
        if node.borrow().is_some() {
            return true;
        }
        let snapshot = gtk::Snapshot::new();
        paintable.snapshot(&snapshot, root.width() as f64, root.height() as f64);
        *node.borrow_mut() = snapshot.to_node();
        node.borrow().is_some()
    });
    let node = node.into_inner().expect("GTK rendered the window");
    root.renderer()
        .unwrap()
        .render_texture(&node, None)
        .save_to_png(PathBuf::from(folder).join(format!("{name}.png")))
        .unwrap();
}
#[test]
#[ignore = "requires GTK display; run under xvfb-run as documented"]
fn native_crud_editor_and_focus() {
    gtk::init().unwrap();
    if std::env::var_os("CCTV_TEST_DARK").is_some() {
        gtk::Settings::default()
            .unwrap()
            .set_gtk_application_prefer_dark_theme(true);
    }
    let app = gtk::Application::builder()
        .application_id("br.dev.schuh.cctv-view.tests")
        .build();
    app.register(None::<&gio::Cancellable>).unwrap();
    let directory = std::env::temp_dir().join(cctv_core::id());
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("test.db");
    Store::open(&path).unwrap();
    let s = build(
        gtk::ApplicationWindow::builder().application(&app).build(),
        path.clone(),
        vec![],
        vec![],
        Preferences::default(),
        Arc::new(tokio::runtime::Runtime::new().unwrap()),
    );
    s.window.present();
    settle();
    capture(&s.window, "01-empty");
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
                    let mut buffer = [0; 2048];
                    let _ = stream.read(&mut buffer);
                    let bytes = include_bytes!("../../media/tests/fixtures/sample.ts");
                    let _=write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: video/mp2t\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",bytes.len());
                    let _ = stream.write_all(bytes);
                });
            } else {
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    });
    assert!(!s.status.is_visible());
    // The empty state leads directly to the real form. Save an ONVIF address without querying it.
    button_named(&s.window, "Adicionar câmera").emit_clicked();
    let form = dialog("Configurar câmera");
    entry_named(&form, "Nome").set_text("Portaria de teste");
    entry_named(&form, "URL do stream").set_text(&stream_url);
    entry_named(&form, "Serviço ONVIF").set_text("http://camera.test/onvif/first");
    capture(&form, "02-camera-connection");
    form.set_default_size(640, 480);
    capture(&form, "03-camera-small");
    let notebook = widgets(&form)
        .into_iter()
        .find_map(|w| w.downcast::<gtk::Notebook>().ok())
        .unwrap();
    let save_bounds = button_named(&form, "Salvar").compute_bounds(&form).unwrap();
    let page_bounds = notebook.compute_bounds(&form).unwrap();
    assert!(
        save_bounds.y() - page_bounds.y() - page_bounds.height() < 20.,
        "Empty feedback must not reserve footer space"
    );
    let kind = widgets(&form)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::DropDown>().ok())
        .find(|w| w.tooltip_text().as_deref() == Some("Tipo de conexão"))
        .unwrap();
    assert_eq!(kind.selected(), 1, "Pasting HTTP chooses HTTP(S)");
    kind.set_selected(2);
    notebook.set_current_page(Some(2));
    capture(&form, "10-mjpeg-options");
    let multipart = widgets(&form)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::DropDown>().ok())
        .find(|w| w.tooltip_text().as_deref() == Some("Formato MJPEG"))
        .unwrap();
    assert!(multipart.is_mapped());
    kind.set_selected(3);
    capture(&form, "11-jpeg-options");
    assert!(!multipart.is_mapped());
    let interval = widgets(&form)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::SpinButton>().ok())
        .find(|w| w.tooltip_text().as_deref() == Some("Atualização JPEG (ms)"))
        .unwrap();
    assert!(interval.is_mapped());
    interval.set_value(700.);
    notebook.set_current_page(Some(1));
    capture(&form, "04-camera-onvif");
    entry_named(&form, "Timeout (ms)").set_text("inválido");
    form.response(gtk::ResponseType::Accept);
    assert_eq!(notebook.current_page(), Some(2));
    capture(&form, "13-footer-error");
    assert!(widgets(&form)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Label>().ok())
        .any(|l| l.text().contains("Timeout deve") && l.is_mapped()));
    entry_named(&form, "Timeout (ms)").set_text("5000");
    form.response(gtk::ResponseType::Accept);
    pump_until(|| s.cameras.borrow().len() == 1);
    let stored = Store::open(&path).unwrap().cameras().unwrap().remove(0);
    assert_eq!(
        stored.capabilities.device_url,
        "http://camera.test/onvif/first"
    );
    assert_eq!(stored.stream_kind, cctv_core::StreamKind::Jpeg);
    assert_eq!(stored.snapshot_interval_ms, 700);
    assert!(!stored.audio_enabled);
    assert_eq!(stored.address, "127.0.0.1");
    // This fixture serves TS video; return it to HTTP(S) before monitoring.
    let mut restored = stored.clone();
    restored.stream_kind = cctv_core::StreamKind::Http;
    Store::open(&path).unwrap().save_camera(&restored).unwrap();
    *s.cameras.borrow_mut() = vec![restored];
    assert_eq!(stored.port, listener_port(&stream_url));
    // Changing the endpoint invalidates capabilities and PTZ authorization, and persists independently.
    let mut configured = s.cameras.borrow()[0].clone();
    configured.capabilities.ptz_url = Some("http://camera.test/ptz".into());
    configured.profile_token = Some("old".into());
    configured.ptz_allowed = true;
    configured.profiles.push(cctv_core::MediaProfile {
        token: "old".into(),
        name: "Old".into(),
        uri: stream_url.clone(),
        width: 320,
        height: 180,
        ptz: true,
    });
    Store::open(&path)
        .unwrap()
        .save_camera(&configured)
        .unwrap();
    *s.cameras.borrow_mut() = vec![configured.clone()];
    camera::form(&s, configured, None);
    let form = dialog("Configurar câmera");
    entry_named(&form, "Serviço ONVIF").set_text("http://new-camera.test/onvif/device_service");
    form.response(gtk::ResponseType::Accept);
    pump_until(|| {
        s.cameras.borrow()[0].capabilities.device_url
            == "http://new-camera.test/onvif/device_service"
    });
    let saved = Store::open(&path).unwrap().cameras().unwrap().remove(0);
    assert!(saved.profiles.is_empty());
    assert!(!saved.ptz_allowed);
    assert!(saved.capabilities.ptz_url.is_none());
    // Click and standard activation are alternatives to drag-and-drop.
    button_named(&s.window, "Criar View").emit_clicked();
    let d = dialog("Editor de Views");
    let name = widgets(&d)
        .into_iter()
        .find_map(|w| w.downcast::<gtk::Entry>().ok())
        .unwrap();
    name.set_text("Entrada");
    let preset = widgets(&d)
        .into_iter()
        .find_map(|w| w.downcast::<gtk::DropDown>().ok())
        .unwrap();
    preset.set_selected(3);
    button_named(&d, "Portaria de teste").emit_clicked();
    button_named(&d, "Desfazer layout").emit_clicked();
    assert!(widgets(&d)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Button>().ok())
        .all(|b| b.label().as_deref() != Some("Portaria de teste\n1 × 1")));
    button_named(&d, "+").emit_clicked();
    let picker = dialog("Escolher câmera");
    button_named(&picker, "Portaria de teste").emit_clicked();
    capture(&d, "05-editor");
    d.set_default_size(720, 500);
    settle();
    assert!(
        name.width() > 300,
        "Name must remain readable in a small editor"
    );
    capture(&d, "06-editor-small");
    preset.set_selected(4);
    assert!(widgets(&d)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Button>().ok())
        .any(|b| b.label().as_deref() == Some("Portaria de teste\n1 × 1")));
    button_named(&d, "Desfazer layout").emit_clicked();
    d.response(gtk::ResponseType::Accept);
    pump_until(|| s.views.borrow().len() == 1);
    let v = s.views.borrow()[0].clone();
    assert_eq!(v.panels.len(), 1);
    assert_eq!((v.columns, v.rows), (2, 2));
    assert_eq!(v.name, "Entrada");
    assert_eq!(Store::open(&path).unwrap().views().unwrap()[0], v);
    // Enter, menu activation, a visible Back button and double-click all reach the same focus state.
    let tile = s.grid.child_at(0, 0).unwrap();
    assert!(tile.grab_focus());
    settle();
    assert!(tile.is_focus());
    let controllers = tile.observe_controllers();
    let keys = (0..controllers.n_items())
        .find_map(|i| {
            controllers
                .item(i)?
                .downcast::<gtk::EventControllerKey>()
                .ok()
        })
        .unwrap();
    use glib::translate::IntoGlib;
    assert!(keys.emit_by_name::<bool>(
        "key-pressed",
        &[
            &gdk::Key::Return.into_glib(),
            &0u32,
            &gdk::ModifierType::empty()
        ]
    ));
    assert!(s.focus.borrow().is_some());
    button_named(&s.window, "Voltar à grade").emit_clicked();
    assert!(s.focus.borrow().is_none());
    button_named(&s.window, "Ampliar câmera").emit_clicked();
    assert!(s.focus.borrow().is_some());
    let controllers = s.window.observe_controllers();
    let keys = (0..controllers.n_items())
        .find_map(|i| {
            controllers
                .item(i)?
                .downcast::<gtk::EventControllerKey>()
                .ok()
        })
        .unwrap();
    assert!(keys.emit_by_name::<bool>(
        "key-pressed",
        &[
            &gdk::Key::Escape.into_glib(),
            &0u32,
            &gdk::ModifierType::empty()
        ]
    ));
    assert!(s.focus.borrow().is_none());
    let camera = s.cameras.borrow()[0].clone();
    let session = s
        .manager
        .lock()
        .unwrap()
        .acquire(&camera, Credentials::default(), false)
        .unwrap();
    pump_until(|| session.latest().is_some());
    settle();
    capture(&s.window, "07-monitor");
    s.window.set_default_size(720, 500);
    pump_until(|| !s.sidebar.is_visible());
    assert!(s.preferences.borrow().sidebar);
    capture(&s.window, "08-monitor-small");
    s.window.set_default_size(1280, 800);
    pump_until(|| s.sidebar.is_visible());
    // Invalid panel edits report errors inside the panel dialog; no work is lost.
    editor::open(&s, v.clone());
    let d = dialog("Editor de Views");
    button_named(&d, "+").emit_clicked();
    button_named(&dialog("Escolher câmera"), "Portaria de teste").emit_clicked();
    button_named(&d, "Portaria de teste\n1 × 1").emit_clicked();
    let panel = dialog("Configurar painel");
    let controls: Vec<_> = widgets(&panel)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::SpinButton>().ok())
        .collect();
    controls[2].set_value(2.);
    controls[3].set_value(2.);
    panel.response(gtk::ResponseType::Accept);
    assert!(panel.is_visible());
    assert!(widgets(&panel)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::Label>().ok())
        .any(|l| l.text().contains("sobrepostos")));
    capture(&panel, "09-panel-error");
    panel.response(gtk::ResponseType::Cancel);
    button_named(&d, "Desfazer layout").emit_clicked();
    // The existing native drop signal remains functional after adding click support.
    let empty = button_named(&d, "+");
    let controllers = empty.observe_controllers();
    let target = (0..controllers.n_items())
        .find_map(|i| controllers.item(i)?.downcast::<gtk::DropTarget>().ok())
        .unwrap();
    let value = glib::BoxedValue(format!("camera:{}", camera.id).to_value());
    assert!(target.emit_by_name::<bool>("drop", &[&value, &0f64, &0f64]));
    button_named(&d, "Desfazer layout").emit_clicked();
    d.response(gtk::ResponseType::Cancel);
    assert_eq!(Store::open(&path).unwrap().views().unwrap()[0], v);
    // Commands are attached to their own View, including initial/favorite and atomic ordering.
    button_named(&s.view_list, "Abrir esta View ao iniciar").emit_clicked();
    assert_eq!(
        s.preferences.borrow().initial_view.as_deref(),
        Some(v.id.as_str())
    );
    button_named(&s.view_list, "Marcar como favorita").emit_clicked();
    pump_until(|| s.views.borrow()[0].favorite);
    *s.focus.borrow_mut() = Some("stale-panel".into());
    render(&s);
    assert!(s.focus.borrow().is_none());
    *s.focus.borrow_mut() = Some(v.panels[0].id.clone());
    render(&s);
    button_named(&s.view_list, "Duplicar").emit_clicked();
    pump_until(|| s.views.borrow().len() == 2);
    let copy_id = s.current.borrow().clone().unwrap();
    assert!(s.focus.borrow().is_none());
    assert!(s.grid.first_child().is_some());
    let original_row = s.view_list.first_child().unwrap();
    button_named(&original_row, "Remover dos favoritos").emit_clicked();
    pump_until(|| {
        !s.views
            .borrow()
            .iter()
            .find(|view| view.id == v.id)
            .unwrap()
            .favorite
    });
    assert_eq!(s.current.borrow().as_deref(), Some(copy_id.as_str()));
    let row = s.view_list.last_child().unwrap();
    button_named(&row, "Mover para cima").emit_clicked();
    pump_until(|| s.views.borrow()[0].id == copy_id);
    assert_eq!(Store::open(&path).unwrap().views().unwrap()[0].id, copy_id);
    // Delete the initial original while another View is active; it must not change selection.
    let row = s.view_list.last_child().unwrap();
    button_named(&row, "Excluir View…").emit_clicked();
    let question = gtk::Window::list_toplevels()
        .into_iter()
        .find_map(|w| w.downcast::<gtk::MessageDialog>().ok())
        .unwrap();
    question.response(gtk::ResponseType::Yes);
    pump_until(|| s.views.borrow().len() == 1);
    assert_eq!(s.current.borrow().as_deref(), Some(copy_id.as_str()));
    assert!(s.preferences.borrow().initial_view.is_none());
    assert!(Store::open(&path).unwrap().cameras().unwrap().len() == 1);
    s.views.borrow_mut()[0].favorite = false;
    Store::open(&path)
        .unwrap()
        .save_view(&s.views.borrow()[0])
        .unwrap();
    refresh_sidebar(&s);
    capture(&s.window, "14-single-view");
    // Compact single-line rows, including badges and a long ellipsized name.
    for (i, name) in [
        "Garagem",
        "Pátio e portões externos — nome muito longo para a lista lateral",
        "Recepção",
    ]
    .into_iter()
    .enumerate()
    {
        let view = View {
            name: name.into(),
            favorite: i == 0,
            order: i as i32 + 1,
            ..Default::default()
        };
        Store::open(&path).unwrap().save_view(&view).unwrap();
    }
    *s.views.borrow_mut() = Store::open(&path).unwrap().views().unwrap();
    refresh_sidebar(&s);
    settle();
    capture(&s.window, "12-view-alignment");
    let rows: Vec<_> = widgets(&s.view_list)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::ListBoxRow>().ok())
        .collect();
    assert_eq!(rows.len(), 4);
    assert!(rows.iter().all(|r| r.height() == rows[0].height()));
    assert!(
        rows.iter().all(|r| r.height() <= 48),
        "Sidebar rows must stay compact"
    );
    let selected = s.view_list.selected_row().unwrap();
    assert_eq!(selected.index(), 0);
    let garage = s.view_list.row_at_index(1).unwrap();
    assert!(garage.grab_focus());
    s.view_list.emit_by_name::<()>("row-activated", &[&garage]);
    assert!(
        garage.is_focus(),
        "Activating a View must preserve keyboard focus on its row"
    );
    assert_eq!(active_view(&s).unwrap().name, "Garagem");
    assert_eq!(s.view_list.selected_row().unwrap().index(), 1);
    ptz_activation_is_bounded(&s);
    s.window.close();
    jobs::flush_preferences();
    server_stop.store(true, std::sync::atomic::Ordering::Release);
    server.join().unwrap();
    std::fs::remove_dir_all(directory).unwrap();
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
fn listener_port(url: &str) -> u16 {
    glib::Uri::parse(url, glib::UriFlags::NONE).unwrap().port() as u16
}
fn ptz_activation_is_bounded(s: &Shared) {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let endpoint = format!("http://{}/onvif", listener.local_addr().unwrap());
    let events = Arc::new(Mutex::new(Vec::<(String, std::time::Instant)>::new()));
    let output = events.clone();
    let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = stop.clone();
    let mock_endpoint = endpoint.clone();
    let worker = std::thread::spawn(move || {
        while !flag.load(std::sync::atomic::Ordering::Acquire) {
            if let Ok((mut stream, _)) = listener.accept() {
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut bytes = vec![];
                let mut buf = [0; 2048];
                loop {
                    let n = stream.read(&mut buf).unwrap_or(0);
                    if n == 0 {
                        break;
                    }
                    bytes.extend_from_slice(&buf[..n]);
                    let text = String::from_utf8_lossy(&bytes);
                    if let Some((headers, body)) = text.split_once("\r\n\r\n") {
                        let length = headers
                            .lines()
                            .find_map(|l| {
                                l.to_ascii_lowercase()
                                    .strip_prefix("content-length: ")
                                    .and_then(|n| n.parse::<usize>().ok())
                            })
                            .unwrap_or(0);
                        if body.len() >= length {
                            break;
                        }
                    }
                }
                let request = String::from_utf8_lossy(&bytes);
                let operation = [
                    "GetDeviceInformation",
                    "GetCapabilities",
                    "GetProfiles",
                    "GetStreamUri",
                    "ContinuousMove",
                    "Stop",
                ]
                .into_iter()
                .find(|op| request.contains(&format!("<op:{op}")))
                .unwrap();
                output
                    .lock()
                    .unwrap()
                    .push((operation.into(), std::time::Instant::now()));
                let payload = match operation {
                    "GetDeviceInformation" => "<Manufacturer>Test</Manufacturer><Model>Local</Model>".to_string(),
                    "GetCapabilities" => format!("<Capabilities><Media><XAddr>{mock_endpoint}</XAddr></Media><PTZ><XAddr>{mock_endpoint}</XAddr></PTZ></Capabilities>"),
                    "GetProfiles" => "<Profiles token=\"one\"><Name>Principal</Name><PTZConfiguration/></Profiles><Profiles token=\"two\"><Name>Secundário</Name><PTZConfiguration/></Profiles>".into(),
                    "GetStreamUri" => {
                        let path = if request.contains(">two</op:ProfileToken>") { "secondary" } else { "primary" };
                        format!("<MediaUri><Uri>rtsp://camera.test/{path}</Uri></MediaUri>")
                    },
                    _ => String::new(),
                };
                let body=format!("<s:Envelope xmlns:s=\"http://www.w3.org/2003/05/soap-envelope\"><s:Body><{operation}Response>{payload}</{operation}Response></s:Body></s:Envelope>");
                let _ = write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
            } else {
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    });
    // A successful query after editing the endpoint must restore profile selection.
    camera::form(s, Camera::default(), None);
    let form = dialog("Configurar câmera");
    entry_named(&form, "Serviço ONVIF").set_text(&endpoint);
    let profiles = widgets(&form)
        .into_iter()
        .filter_map(|w| w.downcast::<gtk::DropDown>().ok())
        .find(|w| w.tooltip_text().as_deref() == Some("Perfil de mídia"))
        .unwrap();
    assert!(!profiles.is_sensitive());
    button_named(&form, "Consultar dispositivo").emit_clicked();
    pump_until(|| profiles.is_sensitive() && profiles.model().is_some_and(|m| m.n_items() == 2));
    profiles.set_selected(1);
    assert_eq!(
        entry_named(&form, "URL do stream").text(),
        "rtsp://camera.test/secondary"
    );
    form.response(gtk::ResponseType::Cancel);
    let c = Camera {
        ptz_allowed: true,
        profile_token: Some("test".into()),
        capabilities: cctv_core::Capabilities {
            device_url: endpoint.clone(),
            ptz_url: Some(endpoint),
            ..Default::default()
        },
        ..Default::default()
    };
    ptz::open(s, &c);
    pump_until(|| {
        gtk::Window::list_toplevels()
            .into_iter()
            .filter_map(|w| w.downcast::<gtk::Dialog>().ok())
            .any(|d| {
                d.title().as_deref() == Some("Controle PTZ")
                    && widgets(&d)
                        .into_iter()
                        .filter_map(|w| w.downcast::<gtk::Button>().ok())
                        .any(|b| b.label().as_deref() == Some("Zoom +"))
            })
    });
    let d = dialog("Controle PTZ");
    button_named(&d, "Zoom +").emit_clicked();
    pump_until(|| {
        let list = events.lock().unwrap();
        list.iter()
            .position(|(op, _)| op == "ContinuousMove")
            .is_some_and(|i| list.iter().skip(i + 1).any(|(op, _)| op == "Stop"))
    });
    {
        let list = events.lock().unwrap();
        let index = list
            .iter()
            .position(|(op, _)| op == "ContinuousMove")
            .unwrap();
        let ended = list
            .iter()
            .skip(index + 1)
            .find(|(op, _)| op == "Stop")
            .unwrap()
            .1;
        assert!(ended.duration_since(list[index].1) < Duration::from_secs(2));
    }
    d.response(gtk::ResponseType::Close);
    settle();
    stop.store(true, std::sync::atomic::Ordering::Release);
    worker.join().unwrap();
}
