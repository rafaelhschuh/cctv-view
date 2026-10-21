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
    fn start(&self, x: f64, y: f64, z: f64) {
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
            let credentials = match camera.secret_ref {
                Some(id) => Vault::get(&id)?,
                None => Credentials::default(),
            };
            cctv_onvif::Client::new(&camera.capabilities.device_url, credentials)
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
            feedback.set_text("Segure uma direção para mover. Solte para parar.");
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
            area.append(&gtk::Label::new(Some("Velocidade")));
            area.append(&speed);
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
                b.set_size_request(70, 42);
                grid.attach(&b, col, row, 1, 1);
                let gesture = gtk::GestureClick::new();
                let c = controller.clone();
                let speed = speed.clone();
                gesture.connect_pressed(move |_, _, _, _| {
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
                gesture.connect_cancel(move |_, _| c.stop());
                b.add_controller(gesture);
            }
            let presets = gtk::DropDown::from_strings(&[]);
            area.append(&presets);
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
            line.append(&name);
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
            keys.connect_key_pressed(move |_, key, _, _| {
                let v = speed2.value();
                let (x, y) = match key {
                    gdk::Key::Up => (0., v),
                    gdk::Key::Down => (0., -v),
                    gdk::Key::Left => (-v, 0.),
                    gdk::Key::Right => (v, 0.),
                    gdk::Key::Escape => {
                        c.stop();
                        return glib::Propagation::Stop;
                    }
                    _ => return glib::Propagation::Proceed,
                };
                if !c.held.load(Ordering::Acquire) {
                    c.start(x, y, 0.);
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
