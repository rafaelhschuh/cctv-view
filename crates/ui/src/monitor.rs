use super::*;
pub(crate) fn render(s: &Shared) {
    let view = active_view(s);
    let valid_focus = s.focus.borrow().as_ref().is_none_or(|id| {
        view.as_ref()
            .is_some_and(|v| v.panels.iter().any(|p| &p.id == id))
    });
    if !valid_focus {
        s.focus.borrow_mut().take();
    }
    if let Some(bar) = s.focus_bar.upgrade() {
        bar.set_visible(s.focus.borrow().is_some());
    }
    if let Ok(m) = s.manager.lock() {
        m.audio.select(None);
    }
    while let Some(c) = s.grid.first_child() {
        s.grid.remove(&c);
    }
    let Some(view) = view else {
        empty(
            s,
            "Seu monitoramento começa aqui",
            "Cadastre uma câmera e crie uma View para acompanhar as imagens.",
        );
        return;
    };
    s.heading.set_text(&view.name);
    s.preferences.borrow_mut().last_view = Some(view.id.clone());
    save_preferences(s);
    if view.panels.is_empty() {
        empty(
            s,
            "Esta View está vazia",
            "Escolha câmeras no editor para compor seu monitoramento.",
        );
        return;
    }
    let focus = s.focus.borrow().clone();
    if focus.is_none() {
        for y in 0..view.rows {
            for x in 0..view.columns {
                if !view
                    .panels
                    .iter()
                    .any(|p| x >= p.x && x < p.x + p.width && y >= p.y && y < p.y + p.height)
                {
                    let placeholder = gtk::Frame::new(None);
                    let label = gtk::Label::new(Some("Painel vazio"));
                    label.add_css_class("dim-label");
                    placeholder.set_child(Some(&label));
                    placeholder.set_hexpand(true);
                    placeholder.set_vexpand(true);
                    s.grid.attach(&placeholder, x as i32, y as i32, 1, 1);
                }
            }
        }
    }
    for panel in &view.panels {
        if focus.as_ref().is_some_and(|id| id != &panel.id) {
            continue;
        }
        let Some(camera) = s
            .cameras
            .borrow()
            .iter()
            .find(|c| c.id == panel.camera_id)
            .cloned()
        else {
            continue;
        };
        let tile = tile(s, &camera, panel.aspect, focus.is_some(), Some(&panel.id));
        let gesture = gtk::GestureClick::new();
        gesture.set_button(1);
        let s2 = s.clone();
        let id = panel.id.clone();
        gesture.connect_pressed(move |_, n, _, _| {
            if n == 2 {
                *s2.focus.borrow_mut() = Some(id.clone());
                render(&s2);
            }
        });
        tile.add_controller(gesture);
        if focus.is_some() {
            s.grid.attach(&tile, 0, 0, 1, 1);
        } else {
            s.grid.attach(
                &tile,
                panel.x as i32,
                panel.y as i32,
                panel.width as i32,
                panel.height as i32,
            );
        }
    }
}
fn empty(s: &Shared, title: &str, subtitle: &str) {
    let b = gtk::Box::new(gtk::Orientation::Vertical, 16);
    b.set_valign(gtk::Align::Center);
    b.set_halign(gtk::Align::Center);
    let icon = gtk::Image::from_icon_name("camera-video-symbolic");
    icon.set_pixel_size(64);
    icon.add_css_class("dim-label");
    b.append(&icon);
    let l = gtk::Label::new(Some(title));
    l.add_css_class("title-2");
    b.append(&l);
    let l = gtk::Label::new(Some(subtitle));
    l.set_wrap(true);
    l.set_max_width_chars(48);
    l.set_justify(gtk::Justification::Center);
    l.add_css_class("dim-label");
    b.append(&l);
    let add = gtk::Button::with_label("Adicionar câmera");
    add.add_css_class("suggested-action");
    let s2 = s.clone();
    add.connect_clicked(move |_| camera::form(&s2, Camera::default(), None));
    b.append(&add);
    let view = active_view(s);
    let next = gtk::Button::with_label(if view.is_some() {
        "Escolher câmeras"
    } else {
        "Criar View"
    });
    let s2 = s.clone();
    next.connect_clicked(move |_| editor::open(&s2, view.clone().unwrap_or_default()));
    b.append(&next);
    s.grid.attach(&b, 0, 0, 1, 1);
}
pub(crate) fn tile(
    s: &Shared,
    camera: &Camera,
    aspect: cctv_core::Aspect,
    focus: bool,
    panel_id: Option<&str>,
) -> gtk::Box {
    let tile = gtk::Box::new(gtk::Orientation::Vertical, 0);
    tile.set_hexpand(true);
    tile.set_vexpand(true);
    tile.add_css_class("card");
    let canvas = player::Canvas::new(aspect);
    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(&canvas));
    overlay.set_vexpand(true);
    tile.append(&overlay);
    let status = gtk::Label::new(Some("Conectando…"));
    status.set_valign(gtk::Align::Center);
    status.set_halign(gtk::Align::Center);
    status.set_wrap(true);
    status.set_max_width_chars(40);
    status.add_css_class("cctv-player-status");
    overlay.add_overlay(&status);
    let bar = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    margins(&bar, 6);
    let name = gtk::Label::new(Some(&camera.name));
    name.set_ellipsize(gtk::pango::EllipsizeMode::End);
    name.set_hexpand(true);
    name.set_xalign(0.);
    bar.append(&name);
    if focus {
        for (icon, label, factor) in [
            ("zoom-out-symbolic", "Diminuir zoom", 1. / 1.25),
            ("zoom-in-symbolic", "Aumentar zoom", 1.25),
            ("zoom-original-symbolic", "Restaurar zoom", 0.),
        ] {
            let b = button(label, icon);
            let weak = canvas.downgrade();
            b.connect_clicked(move |_| {
                if let Some(c) = weak.upgrade() {
                    if factor == 0. {
                        c.reset();
                    } else {
                        c.zoom_by(factor);
                    }
                }
            });
            bar.append(&b);
        }
        if camera.audio_enabled {
            let b = gtk::ToggleButton::builder()
                .icon_name("audio-volume-muted-symbolic")
                .tooltip_text("Áudio desta câmera (exclusivo)")
                .build();
            let manager = s.manager.clone();
            let id = camera.id.clone();
            b.connect_toggled(move |b| {
                if let Ok(m) = manager.lock() {
                    m.audio.select(if b.is_active() {
                        Some(id.clone())
                    } else {
                        None
                    });
                    b.set_icon_name(if b.is_active() {
                        "audio-volume-high-symbolic"
                    } else {
                        "audio-volume-muted-symbolic"
                    });
                }
            });
            bar.append(&b);
        }
        if camera.ptz_allowed
            && camera.capabilities.ptz_url.is_some()
            && camera
                .profiles
                .iter()
                .any(|p| Some(&p.token) == camera.profile_token.as_ref() && p.ptz)
        {
            let b = button("Controles PTZ", "input-gaming-symbolic");
            let s = s.clone();
            let c = camera.clone();
            b.connect_clicked(move |_| ptz::open(&s, &c));
            bar.append(&b);
        }
    }
    let menu = gtk::MenuButton::builder()
        .icon_name("view-more-symbolic")
        .tooltip_text("Ações da câmera")
        .build();
    let popover = gtk::Popover::new();
    let actions = gtk::Box::new(gtk::Orientation::Vertical, 4);
    margins(&actions, 6);
    let configure = gtk::Button::with_label("Configurar câmera");
    let reconnect = gtk::Button::with_label("Reconectar");
    let edit = gtk::Button::with_label("Editar View");
    for button in [&configure, &reconnect, &edit] {
        actions.append(button);
    }
    if let Some(id) = panel_id {
        let enlarge = gtk::Button::with_label(if focus {
            "Voltar à grade"
        } else {
            "Ampliar câmera"
        });
        let s2 = s.clone();
        let id = id.to_string();
        let weak = popover.downgrade();
        enlarge.connect_clicked(move |_| {
            if let Some(p) = weak.upgrade() {
                p.popdown();
            }
            *s2.focus.borrow_mut() = if focus { None } else { Some(id.clone()) };
            render(&s2);
        });
        actions.prepend(&enlarge);
        tile.set_focusable(true);
        tile.update_property(&[gtk::accessible::Property::Label(&format!(
            "Painel da câmera {}. Enter para ampliar.",
            camera.name
        ))]);
        let keys = gtk::EventControllerKey::new();
        let weak_tile = tile.downgrade();
        let state = Rc::downgrade(s);
        let id = panel_id.unwrap_or_default().to_string();
        keys.connect_key_pressed(move |_, key, _, _| {
            if matches!(key, gdk::Key::Return | gdk::Key::KP_Enter)
                && weak_tile.upgrade().is_some_and(|t| t.is_focus())
            {
                if let Some(s) = state.upgrade() {
                    *s.focus.borrow_mut() = if focus { None } else { Some(id.clone()) };
                    render(&s);
                }
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        tile.add_controller(keys);
    }
    popover.set_child(Some(&actions));
    menu.set_popover(Some(&popover));
    bar.append(&menu);
    let s2 = s.clone();
    let c = camera.clone();
    let weak = popover.downgrade();
    configure.connect_clicked(move |_| {
        if let Some(p) = weak.upgrade() {
            p.popdown();
        }
        camera::form(&s2, c.clone(), None);
    });
    let s2 = s.clone();
    let id = camera.id.clone();
    reconnect.connect_clicked(move |_| {
        if let Ok(mut m) = s2.manager.lock() {
            m.invalidate(&id);
        }
        render(&s2);
    });
    let s2 = s.clone();
    let weak = popover.downgrade();
    edit.connect_clicked(move |_| {
        if let Some(p) = weak.upgrade() {
            p.popdown();
        }
        if let Some(v) = active_view(&s2) {
            editor::open(&s2, v);
        }
    });
    let context = gtk::GestureClick::new();
    context.set_button(3);
    let weak = popover.downgrade();
    context.connect_pressed(move |_, _, _, _| {
        if let Some(p) = weak.upgrade() {
            p.popup();
        }
    });
    tile.add_controller(context);
    tile.append(&bar);
    let camera_id = camera.id.clone();
    let state = Rc::downgrade(s);
    let camera = camera.clone();
    let manager = s.manager.clone();
    let weak = canvas.downgrade();
    let status_weak = status.downgrade();
    jobs::run(
        move || -> Result<_> {
            let creds = if camera.no_credentials {
                Credentials::default()
            } else {
                match &camera.secret_ref {
                    Some(id) => Vault::get(id)?,
                    None => Credentials::default(),
                }
            };
            let session = manager
                .lock()
                .map_err(|_| anyhow::anyhow!("Gerenciador indisponível"))?
                .acquire(&camera, creds, focus)?;
            Ok(session)
        },
        move |result| {
            let Some(canvas) = weak.upgrade() else {
                return;
            };
            match result {
                Err(e) => {
                    if let Some(l) = status_weak.upgrade() {
                        l.set_text(&e.to_string());
                    }
                }
                Ok(session) => {
                    let weak = canvas.downgrade();
                    let mut sequence = 0;
                    let mut previous_status = String::new();
                    glib::timeout_add_local(Duration::from_millis(33), move || {
                        let Some(canvas) = weak.upgrade() else {
                            return glib::ControlFlow::Break;
                        };
                        if let Some(frame) = session.latest() {
                            if frame.sequence != sequence {
                                canvas.set_frame(&frame);
                                sequence = frame.sequence;
                            }
                        }
                        let connection_status = match session.status() {
                            Status::Playing => "Conectada",
                            Status::Connecting => "Conectando",
                            Status::Reconnecting { .. } => "Reconectando",
                            Status::Stopped => "Encerrada",
                            Status::Error(_) => "Falha",
                        };
                        if connection_status != previous_status {
                            previous_status = connection_status.into();
                            if let Some(s) = state.upgrade() {
                                if let Some(c) = s
                                    .cameras
                                    .borrow_mut()
                                    .iter_mut()
                                    .find(|c| c.id == camera_id)
                                {
                                    c.last_status = connection_status.into();
                                }
                                let path = s.path.clone();
                                let id = camera_id.clone();
                                let weak = Rc::downgrade(&s);
                                jobs::run(
                                    move || {
                                        Store::open(path)?
                                            .update_camera_status(&id, connection_status)
                                    },
                                    move |r: Result<()>| {
                                        if let (Err(e), Some(s)) = (r, weak.upgrade()) {
                                            s.status.set_text(&format!(
                                                "Falha ao registrar conexão: {e}"
                                            ));
                                        }
                                    },
                                );
                            }
                        }
                        if let Some(l) = status_weak.upgrade() {
                            match session.status() {
                                Status::Playing => l.set_visible(false),
                                Status::Connecting => {
                                    l.set_visible(true);
                                    l.set_text("Conectando…");
                                }
                                Status::Reconnecting { attempt, code } => {
                                    l.set_visible(true);
                                    l.set_text(&format!(
                                        "Sinal indisponível · tentativa {attempt} · código {code}"
                                    ));
                                }
                                Status::Stopped => {
                                    l.set_visible(true);
                                    l.set_text("Sessão encerrada");
                                }
                                Status::Error(e) => {
                                    l.set_visible(true);
                                    l.set_text(&e);
                                }
                            }
                        }
                        glib::ControlFlow::Continue
                    });
                }
            }
        },
    );
    tile
}
