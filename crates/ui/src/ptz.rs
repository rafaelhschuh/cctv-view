use super::*;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
#[derive(Clone)]
struct Controller {
    runtime: Arc<tokio::runtime::Runtime>,
    client: cctv_onvif::Client,
    target: String,
    profile: String,
    held: Arc<AtomicBool>,
    generation: Arc<AtomicU64>,
    serial: Arc<tokio::sync::Mutex<()>>,
    feedback: gtk::Label,
}
impl Controller {
    fn stop(&self) {
        self.held.store(false, Ordering::Release);
        self.generation.fetch_add(1, Ordering::AcqRel);
        let c = self.client.clone();
        let t = self.target.clone();
        let p = self.profile.clone();
        let serial = self.serial.clone();
        self.runtime.spawn(async move {
            let _guard = serial.lock().await;
            let _ = c.stop(&t, &p).await;
        });
    }
    fn start(&self, x: f64, y: f64, z: f64) -> u64 {
        let token = self.generation.fetch_add(1, Ordering::AcqRel) + 1;
        let generation = self.generation.clone();
        // One pulse lasts at most 500ms on the device, renewed only while this control remains held.
        self.held.store(true, Ordering::Release);
        let global = self.held.clone();
        let client = self.client.clone();
        let target = self.target.clone();
        let profile = self.profile.clone();
        let serial = self.serial.clone();
        let runtime = self.runtime.clone();
        let feedback = self.feedback.downgrade();
        jobs::run(
            move || {
                runtime.block_on(async move {
                    let _guard = serial.lock().await;
                    let mut result = Ok(());
                    while global.load(Ordering::Acquire)
                        && generation.load(Ordering::Acquire) == token
                    {
                        result = client.continuous_move(&target, &profile, x, y, z).await;
                        if result.is_err() {
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(200)).await;
                    }
                    let stop = client.stop(&target, &profile).await;
                    result.and(stop)
                })
            },
            move |r: Result<()>| {
                if let (Some(l), Err(e)) = (feedback.upgrade(), r) {
                    l.set_text(&e.to_string());
                }
            },
        );
        token
    }
    /// Keyboard/assistive activation is a bounded pulse, never an unpaired continuous move.
    fn pulse(&self, x: f64, y: f64, z: f64) {
        let token = self.start(x, y, z);
        let c = self.clone();
        glib::timeout_add_local_once(Duration::from_millis(180), move || {
            if c.generation.load(Ordering::Acquire) == token {
                c.stop();
            }
        });
    }
}
pub fn open(s: &Shared, camera: &Camera) {
    if !camera.ptz_allowed {
        return;
    }
    let Some(target) = camera.capabilities.ptz_url.clone() else {
        return;
    };
    let Some(profile) = camera.profile_token.clone() else {
        return;
    };
    let d = gtk::Dialog::builder()
        .transient_for(&s.window)
        .destroy_with_parent(true)
        .title("Controle PTZ")
        .default_width(420)
        .build();
    d.add_button("Fechar", gtk::ResponseType::Close);
    let area = d.content_area();
    margins(&area, 20);
    let feedback = gtk::Label::new(Some("Carregando credenciais…"));
    feedback.set_wrap(true);
    area.append(&feedback);
    let runtime = s.runtime.clone();
    let camera = camera.clone();
    let weak = d.downgrade();
    jobs::run(
        move || -> Result<_> {
            let credentials = if camera.no_credentials {
                Credentials::default()
            } else {
                match camera.secret_ref {
                    Some(id) => Vault::get(&id)?,
                    None => Credentials::default(),
                }
            };
            cctv_onvif::Client::new_with_tls(
                &camera.capabilities.device_url,
                credentials,
                camera.tls_verify,
            )
        },
        move |r| {
            let Some(d) = weak.upgrade() else {
                return;
            };
            let client = match r {
                Ok(c) => c,
                Err(e) => {
                    feedback.set_text(&e.to_string());
                    return;
                }
            };
            feedback.set_text("Mouse: segure para mover e solte para parar. Teclado: ative um botão para um movimento curto. Escape para parar.");
            let controller = Controller {
                runtime,
                client,
                target,
                profile,
                held: Arc::new(AtomicBool::new(false)),
                generation: Arc::new(AtomicU64::new(0)),
                serial: Arc::new(tokio::sync::Mutex::new(())),
                feedback: feedback.clone(),
            };
            let area = d.content_area();
            let grid = gtk::Grid::builder()
                .column_spacing(8)
                .row_spacing(8)
                .halign(gtk::Align::Center)
                .build();
            area.append(&grid);
            let speed = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.1, 1., 0.1);
            speed.set_value(0.4);
            speed.set_tooltip_text(Some("Velocidade PTZ"));
            forms::row(&area, "Velocidade PTZ", &speed);
            for (label, col, row, x, y, z) in [
                ("↖", 0, 0, -1., 1., 0.),
                ("↑", 1, 0, 0., 1., 0.),
                ("↗", 2, 0, 1., 1., 0.),
                ("←", 0, 1, -1., 0., 0.),
                ("■", 1, 1, 0., 0., 0.),
                ("→", 2, 1, 1., 0., 0.),
                ("↙", 0, 2, -1., -1., 0.),
                ("↓", 1, 2, 0., -1., 0.),
                ("↘", 2, 2, 1., -1., 0.),
                ("Zoom −", 0, 3, 0., 0., -1.),
                ("Zoom +", 2, 3, 0., 0., 1.),
            ] {
                let b = gtk::Button::with_label(label);
                let accessible = match label {
                    "↖" => "Mover para cima e esquerda",
                    "↑" => "Mover para cima",
                    "↗" => "Mover para cima e direita",
                    "←" => "Mover para esquerda",
                    "■" => "Parar movimento",
                    "→" => "Mover para direita",
                    "↙" => "Mover para baixo e esquerda",
                    "↓" => "Mover para baixo",
                    "↘" => "Mover para baixo e direita",
                    "Zoom −" => "Diminuir zoom óptico",
                    _ => "Aumentar zoom óptico",
                };
                b.set_tooltip_text(Some(accessible));
                b.update_property(&[gtk::accessible::Property::Label(accessible)]);
                if label == "■" {
                    b.set_label("Parar");
                    b.add_css_class("destructive-action");
                }
                b.set_size_request(70, 44);
                grid.attach(&b, col, row, 1, 1);
                let pointer_used = Rc::new(std::cell::Cell::new(false));
                let gesture = gtk::GestureClick::new();
                gesture.set_button(1);
                let pointer = pointer_used.clone();
                let c = controller.clone();
                let speed = speed.clone();
                let click_speed = speed.clone();
                gesture.connect_pressed(move |_, _, _, _| {
                    pointer.set(true);
                    if x == 0. && y == 0. && z == 0. {
                        c.stop();
                    } else {
                        let v = speed.value();
                        c.start(x * v, y * v, z * v);
                    }
                });
                let c = controller.clone();
                gesture.connect_released(move |_, _, _, _| c.stop());
                let c = controller.clone();
                let pointer = pointer_used.clone();
                gesture.connect_cancel(move |_, _| {
                    pointer.set(false);
                    c.stop();
                });
                let c = controller.clone();
                b.connect_clicked(move |_| {
                    if pointer_used.replace(false) {
                        return;
                    }
                    if x == 0. && y == 0. && z == 0. {
                        c.stop();
                    } else {
                        let v = click_speed.value();
                        c.pulse(x * v, y * v, z * v);
                    }
                });
                b.add_controller(gesture);
            }
            let presets = gtk::DropDown::from_strings(&[]);
            forms::row(&area, "Posição salva", &presets);
            let tokens = Rc::new(RefCell::new(Vec::<cctv_onvif::Preset>::new()));
            let line = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            let load = gtk::Button::with_label("Consultar presets");
            let go = gtk::Button::with_label("Ir ao preset");
            line.append(&load);
            line.append(&go);
            area.append(&line);
            let c = controller.clone();
            let p = presets.clone();
            let list = tokens.clone();
            load.connect_clicked(move |_| {
                let c2 = c.clone();
                let p = p.clone();
                let list = list.clone();
                let rt = c.runtime.clone();
                let client = c.client.clone();
                let target = c.target.clone();
                let profile = c.profile.clone();
                jobs::run(
                    move || rt.block_on(client.presets(&target, &profile)),
                    move |r| match r {
                        Ok(values) => {
                            let labels: Vec<_> = values.iter().map(|v| v.name.as_str()).collect();
                            p.set_model(Some(&gtk::StringList::new(&labels)));
                            *list.borrow_mut() = values;
                        }
                        Err(e) => c2.feedback.set_text(&e.to_string()),
                    },
                );
            });
            let c = controller.clone();
            go.connect_clicked(move |_| {
                let Some(token) = tokens
                    .borrow()
                    .get(presets.selected() as usize)
                    .map(|p| p.token.clone())
                else {
                    return;
                };
                c.stop();
                let rt = c.runtime.clone();
                let client = c.client.clone();
                let target = c.target.clone();
                let profile = c.profile.clone();
                let serial = c.serial.clone();
                let label = c.feedback.clone();
                jobs::run(
                    move || {
                        rt.block_on(async {
                            let _guard = serial.lock().await;
                            client.goto_preset(&target, &profile, &token).await
                        })
                    },
                    move |r: Result<()>| {
                        if let Err(e) = r {
                            label.set_text(&e.to_string());
                        }
                    },
                );
            });
            let line = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            let name = gtk::Entry::builder()
                .placeholder_text("Nome do novo preset")
                .hexpand(true)
                .build();
            let save = gtk::Button::with_label("Salvar posição");
            forms::row(&area, "Nome do novo preset", &name);
            line.append(&save);
            area.append(&line);
            let c = controller.clone();
            save.connect_clicked(move |_| {
                let name = name.text().to_string();
                let client = c.client.clone();
                let target = c.target.clone();
                let profile = c.profile.clone();
                let rt = c.runtime.clone();
                let label = c.feedback.clone();
                jobs::run(
                    move || rt.block_on(client.set_preset(&target, &profile, &name)),
                    move |r| {
                        label.set_text(&match r {
                            Ok(_) => "Preset salvo. Consulte a lista para atualizar.".into(),
                            Err(e) => e.to_string(),
                        })
                    },
                );
            });
            let c = controller.clone();
            d.connect_is_active_notify(move |d| {
                if !d.is_active() {
                    c.stop();
                }
            });
            let c = controller.clone();
            d.connect_close_request(move |_| {
                c.stop();
                glib::Propagation::Proceed
            });
            let c = controller.clone();
            d.connect_response(move |d, _| {
                c.stop();
                d.close();
            });
            let keys = gtk::EventControllerKey::new();
            let c = controller.clone();
            let speed2 = speed.clone();
            let weak_window = d.downgrade();
            keys.connect_key_pressed(move |_, key, _, _| {
                if key != gdk::Key::Escape
                    && weak_window
                        .upgrade()
                        .and_then(|d| gtk::prelude::RootExt::focus(&d))
                        .is_some_and(|w| {
                            std::iter::successors(Some(w), |w| w.parent()).any(|w| {
                                w.is::<gtk::Entry>()
                                    || w.is::<gtk::SpinButton>()
                                    || w.is::<gtk::Scale>()
                                    || w.is::<gtk::DropDown>()
                            })
                        })
                {
                    return glib::Propagation::Proceed;
                }
                let v = speed2.value();
                let (x, y, z) = match key {
                    gdk::Key::Up => (0., v, 0.),
                    gdk::Key::Down => (0., -v, 0.),
                    gdk::Key::Left => (-v, 0., 0.),
                    gdk::Key::Right => (v, 0., 0.),
                    gdk::Key::Home => (-v, v, 0.),
                    gdk::Key::Page_Up => (v, v, 0.),
                    gdk::Key::End => (-v, -v, 0.),
                    gdk::Key::Page_Down => (v, -v, 0.),
                    gdk::Key::plus | gdk::Key::KP_Add => (0., 0., v),
                    gdk::Key::minus | gdk::Key::KP_Subtract => (0., 0., -v),
                    gdk::Key::Escape => {
                        c.stop();
                        return glib::Propagation::Stop;
                    }
                    _ => return glib::Propagation::Proceed,
                };
                if !c.held.load(Ordering::Acquire) {
                    c.start(x, y, z);
                }
                glib::Propagation::Stop
            });
            let c = controller.clone();
            keys.connect_key_released(move |_, _, _, _| c.stop());
            d.add_controller(keys);
        },
    );
    d.present();
}
