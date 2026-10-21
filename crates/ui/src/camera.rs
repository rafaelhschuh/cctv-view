use super::*;
use cctv_core::{Protocol, Transport};
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
fn field(grid: &gtk::Grid, row: i32, label: &str, value: &str) -> gtk::Entry {
    let l = gtk::Label::new(Some(label));
    l.set_xalign(0.);
    let e = gtk::Entry::builder().text(value).hexpand(true).build();
    e.set_tooltip_text(Some(label));
    grid.attach(&l, 0, row, 1, 1);
    grid.attach(&e, 1, row, 1, 1);
    e
}
pub fn form(s: &Shared, camera: Camera, library: Option<gtk::Box>) {
    let d = gtk::Dialog::builder()
        .transient_for(&s.window)
        .destroy_with_parent(true)
        .modal(true)
        .title("Configurar câmera")
        .default_width(680)
        .default_height(700)
        .build();
    d.add_button("Cancelar", gtk::ResponseType::Cancel);
    let save = d.add_button("Salvar", gtk::ResponseType::Accept);
    save.add_css_class("suggested-action");
    let scroll = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();
    d.content_area().append(&scroll);
    let grid = gtk::Grid::builder()
        .column_spacing(16)
        .row_spacing(12)
        .build();
    margins(&grid, 20);
    scroll.set_child(Some(&grid));
    let original = Rc::new(RefCell::new(camera.clone()));
    let name = field(&grid, 0, "Nome", &camera.name);
    let stream = field(&grid, 1, "URL do stream", &camera.stream_url);
    stream.set_placeholder_text(Some("rtsp://192.168.1.20:554/live"));
    let sub = field(
        &grid,
        2,
        "URL do substream",
        camera.substream_url.as_deref().unwrap_or(""),
    );
    let address = field(&grid, 3, "Endereço", &camera.address);
    let port = field(&grid, 4, "Porta", &camera.port.to_string());
    let user = field(&grid, 5, "Usuário", "");
    let password = gtk::PasswordEntry::builder()
        .show_peek_icon(true)
        .hexpand(true)
        .build();
    password.set_placeholder_text(Some("Em branco: manter credencial atual"));
    grid.attach(&gtk::Label::new(Some("Senha (chaveiro)")), 0, 6, 1, 1);
    grid.attach(&password, 1, 6, 1, 1);
    let forget = gtk::CheckButton::with_label("Remover credenciais salvas");
    grid.attach(&forget, 1, 7, 1, 1);
    let transport = gtk::DropDown::from_strings(&["RTSP por TCP", "RTSP por UDP"]);
    transport.set_selected(if camera.transport == Transport::Tcp {
        0
    } else {
        1
    });
    grid.attach(&gtk::Label::new(Some("Transporte")), 0, 8, 1, 1);
    grid.attach(&transport, 1, 8, 1, 1);
    let timeout = field(&grid, 9, "Timeout (ms)", &camera.timeout_ms.to_string());
    let audio = gtk::CheckButton::with_label("Permitir áudio no modo foco");
    audio.set_active(camera.audio_enabled);
    grid.attach(&audio, 1, 10, 1, 1);
    let hardware = gtk::CheckButton::with_label("Tentar aceleração VA-API; fallback por software");
    hardware.set_active(camera.hardware_decode);
    grid.attach(&hardware, 1, 11, 1, 1);
    let endpoint = field(&grid, 12, "Serviço ONVIF", &camera.capabilities.device_url);
    endpoint.set_placeholder_text(Some("http://192.168.1.20/onvif/device_service"));
    let query = gtk::Button::with_label("Consultar dispositivo e perfis ONVIF");
    grid.attach(&query, 1, 13, 1, 1);
    let profiles = gtk::DropDown::from_strings(
        &camera
            .profiles
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>(),
    );
    grid.attach(&gtk::Label::new(Some("Perfil de mídia")), 0, 14, 1, 1);
    grid.attach(&profiles, 1, 14, 1, 1);
    if let Some(index) = camera
        .profiles
        .iter()
        .position(|p| Some(&p.token) == camera.profile_token.as_ref())
    {
        profiles.set_selected(index as u32);
    }
    let ptz = gtk::CheckButton::with_label("Autorizar controle PTZ deste dispositivo");
    ptz.set_active(camera.ptz_allowed);
    ptz.set_sensitive(camera.capabilities.ptz_url.is_some());
    grid.attach(&ptz, 1, 15, 1, 1);
    let test = gtk::Button::with_label("Testar conexão de vídeo");
    grid.attach(&test, 1, 16, 1, 1);
    let feedback = gtk::Label::new(None);
    feedback.set_wrap(true);
    feedback.set_max_width_chars(55);
    feedback.set_xalign(0.);
    grid.attach(&feedback, 0, 17, 2, 1);
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
                    user.set_text(&c.username);
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
                        p.set_selected(0);
                        ptz.set_sensitive(orig.borrow().capabilities.ptz_url.is_some());
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
        let address = address.clone();
        let port = port.clone();
        let timeout = timeout.clone();
        let transport = transport.clone();
        let audio = audio.clone();
        let hardware = hardware.clone();
        let ptz = ptz.clone();
        let profiles = profiles.clone();
        move || -> Result<Camera> {
            let mut c = original.borrow().clone();
            c.name = name.text().to_string();
            c.stream_url = stream.text().to_string();
            c.substream_url = if sub.text().is_empty() {
                None
            } else {
                Some(sub.text().to_string())
            };
            c.address = address.text().to_string();
            c.port = port
                .text()
                .parse()
                .map_err(|_| anyhow::anyhow!("Porta inválida"))?;
            c.timeout_ms = timeout
                .text()
                .parse()
                .map_err(|_| anyhow::anyhow!("Timeout inválido"))?;
            c.transport = if transport.selected() == 0 {
                Transport::Tcp
            } else {
                Transport::Udp
            };
            c.audio_enabled = audio.is_active();
            c.hardware_decode = hardware.is_active();
            c.ptz_allowed = ptz.is_active();
            c.profile_token = c
                .profiles
                .get(profiles.selected() as usize)
                .map(|p| p.token.clone());
            c.protocol = if !c.capabilities.device_url.is_empty() {
                Protocol::Onvif
            } else if c.stream_url.starts_with("https:") {
                Protocol::Https
            } else if c.stream_url.starts_with("http:") {
                Protocol::Http
            } else {
                Protocol::Rtsp
            };
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
                            c.capabilities.device_url = device.endpoint.clone();
                            if let Ok(u) = url_from_endpoint(&device.endpoint) {
                                c.address = u.0;
                                c.port = u.1;
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
fn url_from_endpoint(value: &str) -> Result<(String, u16)> {
    let u = gtk::glib::Uri::parse(value, gtk::glib::UriFlags::NONE)?;
    Ok((
        u.host().map(|s| s.to_string()).unwrap_or_default(),
        if u.port() > 0 { u.port() as u16 } else { 80 },
    ))
}
