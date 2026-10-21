use super::*;
use cctv_core::Transport;
use std::cell::Cell;

pub fn library(s: &Shared) {
    let d = gtk::Dialog::builder()
        .transient_for(&s.window)
        .destroy_with_parent(true)
        .title("Biblioteca de câmeras")
        .default_width(700)
        .default_height(500)
        .build();
    d.add_button("Fechar", gtk::ResponseType::Close);
    let content = d.content_area();
    margins(&content, 16);
    let top = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let add = gtk::Button::with_label("Adicionar câmera");
    add.add_css_class("suggested-action");
    let discover = gtk::Button::with_label("Descobrir ONVIF");
    top.append(&add);
    top.append(&discover);
    content.append(&top);
    let list = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let scroll = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .child(&list)
        .build();
    content.append(&scroll);
    populate(s, &list);
    let s2 = s.clone();
    let list2 = list.clone();
    add.connect_clicked(move |_| form(&s2, Camera::default(), Some(list2.clone())));
    let s2 = s.clone();
    let list2 = list.clone();
    discover.connect_clicked(move |_| discovery(&s2, list2.clone()));
    let list_for_close = list.clone();
    d.connect_close_request(move |_| {
        while let Some(child) = list_for_close.first_child() {
            list_for_close.remove(&child);
        }
        glib::Propagation::Proceed
    });
    d.connect_response(move |d, _| {
        while let Some(child) = list.first_child() {
            list.remove(&child);
        }
        d.close();
    });
    d.present();
}
fn populate(s: &Shared, list: &gtk::Box) {
    while let Some(c) = list.first_child() {
        list.remove(&c);
    }
    if s.cameras.borrow().is_empty() {
        let l = gtk::Label::new(Some(
            "Nenhuma câmera cadastrada. Adicione pela URL ou descubra dispositivos ONVIF na rede.",
        ));
        l.set_wrap(true);
        margins(&l, 24);
        list.append(&l);
    }
    for c in s.cameras.borrow().iter() {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        margins(&row, 8);
        let l = gtk::Label::new(Some(&format!(
            "{}\n{} {} · {}",
            c.name, c.manufacturer, c.model, c.last_status
        )));
        l.set_xalign(0.);
        l.set_hexpand(true);
        l.set_ellipsize(gtk::pango::EllipsizeMode::End);
        row.append(&l);
        let edit = button("Editar câmera", "document-edit-symbolic");
        let delete = button("Excluir câmera", "user-trash-symbolic");
        row.append(&edit);
        row.append(&delete);
        list.append(&row);
        let s2 = s.clone();
        let c2 = c.clone();
        let l2 = list.clone();
        edit.connect_clicked(move |_| form(&s2, c2.clone(), Some(l2.clone())));
        let s2 = s.clone();
        let c = c.clone();
        let l2 = list.clone();
        delete.connect_clicked(move |_| {
            let s3 = s2.clone();
            let c = c.clone();
            let l = l2.clone();
            confirm(
                &s2.window,
                "Excluir a câmera e removê-la de todas as Views?",
                move || {
                    let path = s3.path.clone();
                    let s4 = s3.clone();
                    jobs::run(
                        move || -> Result<_> {
                            if let Some(secret) = &c.secret_ref {
                                Vault::delete(secret)?;
                            }
                            let mut store = Store::open(path)?;
                            store.delete_camera(&c.id)?;
                            Ok((store.cameras()?, store.views()?))
                        },
                        move |r| match r {
                            Ok((c, v)) => {
                                *s4.cameras.borrow_mut() = c;
                                *s4.views.borrow_mut() = v;
                                populate(&s4, &l);
                                refresh(&s4);
                            }
                            Err(e) => message(&s4.window, &e.to_string()),
                        },
                    );
                },
            );
        });
    }
}
pub fn form(s: &Shared, camera: Camera, library: Option<gtk::Box>) {
    let d = gtk::Dialog::builder()
        .transient_for(&s.window)
        .destroy_with_parent(true)
        .modal(true)
        .title("Configurar câmera")
        .default_width(640)
        .default_height(640)
        .build();
    d.add_button("Cancelar", gtk::ResponseType::Cancel);
    let save = d.add_button("Salvar", gtk::ResponseType::Accept);
    save.add_css_class("suggested-action");
    d.set_default_response(gtk::ResponseType::Accept);
    let notebook = gtk::Notebook::new();
    notebook.set_vexpand(true);
    notebook.set_scrollable(true);
    d.content_area().append(&notebook);
    let connection = forms::page(&notebook, "Conexão");
    let onvif = forms::page(&notebook, "ONVIF");
    let advanced = forms::page(&notebook, "Avançado");
    let original = Rc::new(RefCell::new(camera.clone()));
    forms::section(&connection, "Identificação");
    let name = forms::entry(&connection, "Nome", &camera.name);
    forms::section(&connection, "Conexão de vídeo");
    let stream = forms::entry(&connection, "URL do stream", &camera.stream_url);
    stream.set_placeholder_text(Some("rtsp://192.168.1.20:554/live"));
    forms::help(&connection,"Informe a URL completa do vídeo, incluindo a porta quando necessária. Usuário e senha ficam nos campos abaixo.");
    let destination = gtk::Label::new(None);
    destination.set_xalign(0.);
    destination.set_wrap(true);
    destination.add_css_class("dim-label");
    connection.append(&destination);
    let update_destination = {
        let destination = destination.clone();
        let transport_state = camera.clone();
        move |entry: &gtk::Entry| {
            let mut c = transport_state.clone();
            destination.set_text(&if c.set_stream_url(&entry.text()).is_ok() {
                format!("Destino do vídeo: {} · porta {}", c.address, c.port)
            } else {
                "Destino definido pela URL do vídeo.".into()
            });
        }
    };
    update_destination(&stream);
    stream.connect_changed(update_destination);
    forms::section(&connection, "Credenciais");
    let user = forms::entry(&connection, "Usuário", "");
    let password = gtk::PasswordEntry::builder()
        .show_peek_icon(true)
        .hexpand(true)
        .build();
    password.set_placeholder_text(Some(if camera.secret_ref.is_some() {
        "Em branco: manter a senha salva"
    } else {
        "Senha da câmera"
    }));
    forms::row(&connection, "Senha", &password);
    forms::help(
        &connection,
        "As credenciais são protegidas pelo chaveiro do sistema.",
    );
    let test = gtk::Button::with_label("Testar conexão de vídeo");
    test.set_halign(gtk::Align::Start);
    connection.append(&test);
    forms::section(&onvif, "Descoberta e controle");
    forms::help(&onvif,"Opcional para câmeras cadastradas por URL. Consulte o dispositivo para obter perfis de vídeo e verificar o controle PTZ.");
    let endpoint = forms::entry(&onvif, "Serviço ONVIF", &camera.capabilities.device_url);
    endpoint.set_placeholder_text(Some("http://192.168.1.20/onvif/device_service"));
    let query = gtk::Button::with_label("Consultar dispositivo");
    query.set_halign(gtk::Align::Start);
    onvif.append(&query);
    let profiles = gtk::DropDown::from_strings(
        &camera
            .profiles
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>(),
    );
    forms::row(&onvif, "Perfil de mídia", &profiles);
    profiles.set_sensitive(!camera.profiles.is_empty());
    if let Some(i) = camera
        .profiles
        .iter()
        .position(|p| Some(&p.token) == camera.profile_token.as_ref())
    {
        profiles.set_selected(i as u32);
    }
    let ptz = gtk::CheckButton::with_label("Permitir controle PTZ desta câmera");
    ptz.set_active(camera.ptz_allowed);
    ptz.set_sensitive(camera.capabilities.ptz_url.is_some());
    onvif.append(&ptz);
    forms::help(&onvif,"Ao mudar o endereço ONVIF, consulte novamente o dispositivo antes de habilitar PTZ. O endereço digitado será salvo mesmo sem consulta.");
    forms::section(&advanced, "Reprodução");
    let sub = forms::entry(
        &advanced,
        "URL do substream",
        camera.substream_url.as_deref().unwrap_or(""),
    );
    forms::help(
        &advanced,
        "Opcional: use um stream de menor resolução no mosaico. O foco utiliza o vídeo principal.",
    );
    let transport = gtk::DropDown::from_strings(&["TCP — recomendado", "UDP"]);
    transport.set_selected(if camera.transport == Transport::Tcp {
        0
    } else {
        1
    });
    forms::row(&advanced, "Transporte RTSP", &transport);
    transport.set_sensitive(camera.stream_url.is_empty() || camera.stream_url.starts_with("rtsp:"));
    let weak = transport.downgrade();
    stream.connect_changed(move |e| {
        if let Some(t) = weak.upgrade() {
            t.set_sensitive(e.text().starts_with("rtsp:"));
        }
    });
    let timeout = forms::entry(&advanced, "Timeout (ms)", &camera.timeout_ms.to_string());
    let audio = gtk::CheckButton::with_label("Permitir áudio no modo foco");
    audio.set_active(camera.audio_enabled);
    advanced.append(&audio);
    let hardware = gtk::CheckButton::with_label("Usar aceleração de vídeo quando disponível");
    hardware.set_active(camera.hardware_decode);
    advanced.append(&hardware);
    forms::help(
        &advanced,
        "Se a aceleração não estiver disponível, a reprodução usa o processador automaticamente.",
    );
    forms::section(&advanced, "Credenciais salvas");
    let forget = gtk::CheckButton::with_label("Remover credenciais do chaveiro ao salvar");
    forget.set_sensitive(camera.secret_ref.is_some());
    advanced.append(&forget);
    let feedback = gtk::Label::new(None);
    feedback.set_wrap(true);
    feedback.set_max_width_chars(64);
    feedback.set_xalign(0.);
    feedback.set_size_request(-1, 36);
    margins(&feedback, 12);
    feedback.update_property(&[gtk::accessible::Property::Label(
        "Resultado da configuração",
    )]);
    d.content_area().append(&feedback);
    let loaded = Rc::new(RefCell::new(None::<Credentials>));
    let busy = Rc::new(Cell::new(false));
    if let Some(id) = camera.secret_ref.clone() {
        let user = user.clone();
        let loaded = loaded.clone();
        let feedback = feedback.clone();
        jobs::run(
            move || Vault::get(&id),
            move |r| match r {
                Ok(c) => {
                    if user.text().is_empty() {
                        user.set_text(&c.username);
                    }
                    *loaded.borrow_mut() = Some(c);
                }
                Err(e) => feedback.set_text(&e.to_string()),
            },
        );
    }
    let get_creds = {
        let user = user.clone();
        let password = password.clone();
        let loaded = loaded.clone();
        let forget = forget.clone();
        move || {
            if forget.is_active() {
                Credentials::default()
            } else {
                Credentials {
                    username: user.text().to_string(),
                    password: if password.text().is_empty() {
                        loaded
                            .borrow()
                            .as_ref()
                            .map(|c| c.password.clone())
                            .unwrap_or_default()
                    } else {
                        password.text().to_string()
                    },
                }
            }
        }
    };
    let get_creds = Rc::new(get_creds);
    let original2 = original.clone();
    let stream2 = stream.clone();
    let sub2 = sub.clone();
    let ptz2 = ptz.clone();
    profiles.connect_selected_notify(move |p| {
        let c = original2.borrow();
        if let Some(profile) = c.profiles.get(p.selected() as usize) {
            stream2.set_text(&profile.uri);
            ptz2.set_sensitive(profile.ptz && c.capabilities.ptz_url.is_some());
            if !profile.ptz {
                ptz2.set_active(false);
            }
            if let Some(small) = c
                .profiles
                .iter()
                .filter(|p| p.width > 0)
                .min_by_key(|p| u64::from(p.width) * u64::from(p.height))
            {
                if small.token != profile.token {
                    sub2.set_text(&small.uri);
                }
            }
        }
    });
    let orig_endpoint = original.clone();
    let profiles2 = profiles.clone();
    let ptz2 = ptz.clone();
    endpoint.connect_changed(move |entry| {
        let unchanged = entry.text().as_str() == orig_endpoint.borrow().capabilities.device_url;
        profiles2.set_sensitive(unchanged);
        ptz2.set_sensitive(unchanged && orig_endpoint.borrow().capabilities.ptz_url.is_some());
        if !unchanged {
            ptz2.set_active(false);
        }
    });
    let runtime = s.runtime.clone();
    let get = get_creds.clone();
    let orig = original.clone();
    let p = profiles.clone();
    let feed = feedback.clone();
    let endpoint2 = endpoint.clone();
    let busy2 = busy.clone();
    let ptz2 = ptz.clone();
    query.connect_clicked(move |b| {
        if busy2.replace(true) {
            return;
        }
        b.set_sensitive(false);
        feed.set_text("Consultando dispositivo…");
        let credentials = get();
        let endpoint = endpoint2.text().to_string();
        let requested_endpoint = endpoint.clone();
        let endpoint_widget = endpoint2.clone();
        let rt = runtime.clone();
        let orig = orig.clone();
        let p = p.clone();
        let feed = feed.clone();
        let busy = busy2.clone();
        let button = b.downgrade();
        let ptz = ptz2.clone();
        jobs::run(
            move || {
                rt.block_on(async {
                    cctv_onvif::Client::new(&endpoint, credentials)?
                        .inspect()
                        .await
                })
            },
            move |r| {
                busy.set(false);
                if let Some(b) = button.upgrade() {
                    b.set_sensitive(true);
                }
                if endpoint_widget.text().as_str() != requested_endpoint {
                    feed.set_text("O endereço foi alterado. Consulte o novo dispositivo.");
                    return;
                }
                match r {
                    Ok(info) => {
                        feed.set_text(&format!(
                            "{} {} · {} perfis",
                            info.manufacturer,
                            info.model,
                            info.profiles.len()
                        ));
                        let labels: Vec<_> = info
                            .profiles
                            .iter()
                            .map(|p| {
                                format!(
                                    "{} · {}×{}{}",
                                    p.name,
                                    p.width,
                                    p.height,
                                    if p.ptz { " · PTZ" } else { "" }
                                )
                            })
                            .collect();
                        let model = gtk::StringList::new(
                            &labels.iter().map(String::as_str).collect::<Vec<_>>(),
                        );
                        {
                            let mut c = orig.borrow_mut();
                            c.manufacturer = info.manufacturer;
                            c.model = info.model;
                            c.capabilities = info.capabilities;
                            c.profiles = info.profiles;
                        }
                        p.set_model(Some(&model));
                        p.set_sensitive(true);
                        p.set_selected(gtk::INVALID_LIST_POSITION);
                        p.set_selected(0);
                        let c = orig.borrow();
                        ptz.set_sensitive(
                            c.capabilities.ptz_url.is_some()
                                && c.profiles.first().is_some_and(|p| p.ptz),
                        );
                    }
                    Err(e) => feed.set_text(&e.to_string()),
                }
            },
        );
    });
    let collect = {
        let original = original.clone();
        let name = name.clone();
        let stream = stream.clone();
        let sub = sub.clone();
        let endpoint = endpoint.clone();
        let notebook = notebook.clone();
        let timeout = timeout.clone();
        let transport = transport.clone();
        let audio = audio.clone();
        let hardware = hardware.clone();
        let ptz = ptz.clone();
        let profiles = profiles.clone();
        move || -> Result<Camera> {
            let mut c = original.borrow().clone();
            c.name = name.text().trim().to_string();
            if c.name.is_empty() || c.name.len() > 160 {
                notebook.set_current_page(Some(0));
                name.grab_focus();
                anyhow::bail!("Informe um nome com até 160 caracteres.");
            }
            if let Err(e) = c.set_onvif_endpoint(&endpoint.text()) {
                notebook.set_current_page(Some(1));
                endpoint.grab_focus();
                return Err(e);
            }
            if let Err(e) = c.set_stream_url(&stream.text()) {
                notebook.set_current_page(Some(0));
                stream.grab_focus();
                return Err(e);
            }
            c.substream_url = if sub.text().trim().is_empty() {
                None
            } else {
                let value = sub.text().trim().to_string();
                if let Err(e) = cctv_core::validate_stream(&value) {
                    notebook.set_current_page(Some(2));
                    sub.grab_focus();
                    return Err(e);
                }
                Some(value)
            };
            c.timeout_ms = match timeout.text().trim().parse::<u32>() {
                Ok(ms) if (500..=60000).contains(&ms) => ms,
                _ => {
                    notebook.set_current_page(Some(2));
                    timeout.grab_focus();
                    anyhow::bail!("Timeout deve ficar entre 500 e 60000 ms.");
                }
            };
            c.transport = if transport.selected() == 0 {
                Transport::Tcp
            } else {
                Transport::Udp
            };
            c.audio_enabled = audio.is_active();
            c.hardware_decode = hardware.is_active();
            c.ptz_allowed = ptz.is_active() && c.capabilities.ptz_url.is_some();
            c.profile_token = c
                .profiles
                .get(profiles.selected() as usize)
                .map(|p| p.token.clone());
            c.validate()?;
            Ok(c)
        }
    };
    let collect = Rc::new(collect);
    let gather = collect.clone();
    let get = get_creds.clone();
    let feed = feedback.clone();
    let busy2 = busy.clone();
    test.connect_clicked(move |b| {
        if busy2.get() {
            return;
        }
        match gather() {
            Err(e) => feed.set_text(&e.to_string()),
            Ok(mut c) => {
                busy2.set(true);
                b.set_sensitive(false);
                feed.set_text("Aguardando um frame real do stream…");
                let creds = get();
                let feed = feed.clone();
                let busy = busy2.clone();
                let button = b.downgrade();
                let orig = original.clone();
                jobs::run(
                    move || {
                        let result = cctv_media::connection_test(&c, creds);
                        c.last_status = if result.is_ok() {
                            "Conexão verificada".into()
                        } else {
                            "Falha na conexão".into()
                        };
                        (result, c.last_status)
                    },
                    move |(r, status)| {
                        orig.borrow_mut().last_status = status;
                        busy.set(false);
                        if let Some(b) = button.upgrade() {
                            b.set_sensitive(true);
                        }
                        feed.set_text(&match r {
                            Ok(()) => "Vídeo recebido. Conexão verificada.".into(),
                            Err(e) => e.to_string(),
                        });
                    },
                );
            }
        }
    });
    let s = s.clone();
    d.connect_response(move |d, r| {
        if r != gtk::ResponseType::Accept {
            d.close();
            return;
        }
        if busy.get() {
            feedback.set_text("Aguarde a operação em andamento.");
            return;
        }
        let mut c = match collect() {
            Ok(c) => c,
            Err(e) => {
                feedback.set_text(&e.to_string());
                return;
            }
        };
        let creds = get_creds();
        let forget = forget.is_active();
        if !forget
            && c.secret_ref.is_some()
            && loaded.borrow().is_none()
            && password.text().is_empty()
        {
            feedback.set_text("Desbloqueie o chaveiro ou informe novamente a senha.");
            return;
        }
        busy.set(true);
        save.set_sensitive(false);
        let path = s.path.clone();
        let s2 = s.clone();
        let d2 = d.downgrade();
        let feedback = feedback.clone();
        let library = library.clone();
        let busy = busy.clone();
        let save = save.clone();
        jobs::run(
            move || -> Result<_> {
                let previous = c.secret_ref.clone();
                if forget || creds.username.is_empty() {
                    if let Some(id) = &previous {
                        Vault::delete(id)?;
                    }
                    c.secret_ref = None;
                } else {
                    let reference = format!("camera:{}", c.id);
                    Vault::store(&reference, &creds)?;
                    c.secret_ref = Some(reference);
                }
                let store = Store::open(path)?;
                store.save_camera(&c)?;
                Ok((store.cameras()?, c.id))
            },
            move |r| {
                busy.set(false);
                save.set_sensitive(true);
                match r {
                    Ok((cameras, id)) => {
                        if let Ok(mut manager) = s2.manager.lock() {
                            manager.invalidate(&id);
                        }
                        *s2.cameras.borrow_mut() = cameras;
                        if let Some(list) = library {
                            populate(&s2, &list);
                        }
                        refresh(&s2);
                        if let Some(d) = d2.upgrade() {
                            d.close();
                        }
                    }
                    Err(e) => feedback.set_text(&e.to_string()),
                }
            },
        );
    });
    d.present();
}
fn discovery(s: &Shared, library: gtk::Box) {
    let d = gtk::Dialog::builder()
        .transient_for(&s.window)
        .destroy_with_parent(true)
        .title("Descoberta ONVIF na rede local")
        .default_width(620)
        .default_height(400)
        .build();
    d.add_button("Fechar", gtk::ResponseType::Close);
    let area = d.content_area();
    margins(&area, 16);
    let label = gtk::Label::new(Some("Procurando dispositivos por 4 segundos…"));
    area.append(&label);
    let list = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let scroll = gtk::ScrolledWindow::builder()
        .child(&list)
        .vexpand(true)
        .build();
    area.append(&scroll);
    let runtime = s.runtime.clone();
    let s = s.clone();
    let weak = d.downgrade();
    jobs::run(
        move || runtime.block_on(cctv_onvif::discover(Duration::from_secs(4))),
        move |r| {
            if weak.upgrade().is_none() {
                return;
            }
            match r {
                Ok(devices) => {
                    label.set_text(if devices.is_empty(){"Nenhum dispositivo respondeu. Verifique a rede e o firewall ou cadastre o endpoint manualmente."}else{"Selecione um dispositivo e consulte os perfis com suas credenciais."});
                    for device in devices {
                        let b = gtk::Button::with_label(&device.endpoint);
                        b.set_tooltip_text(Some(&device.scopes));
                        list.append(&b);
                        let s = s.clone();
                        let library = library.clone();
                        b.connect_clicked(move |_| {
                            let mut c = Camera::default();
                            if let Err(e) = c.set_onvif_endpoint(&device.endpoint) {
                                message(&s.window, &e.to_string());
                                return;
                            }
                            form(&s, c, Some(library.clone()));
                        });
                    }
                }
                Err(e) => label.set_text(&format!("Falha na descoberta: {e}")),
            }
        },
    );
    d.connect_response(|d, _| d.close());
    d.present();
}
