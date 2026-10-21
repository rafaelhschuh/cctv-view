use super::*;
pub fn open(s: &Shared) {
    let d = gtk::Window::builder()
        .transient_for(&s.window)
        .destroy_with_parent(true)
        .title("Diagnóstico de sessões")
        .default_width(750)
        .default_height(420)
        .build();
    let text = gtk::TextView::builder()
        .editable(false)
        .cursor_visible(false)
        .monospace(true)
        .left_margin(16)
        .right_margin(16)
        .top_margin(16)
        .build();
    let scroll = gtk::ScrolledWindow::builder().child(&text).build();
    d.set_child(Some(&scroll));
    d.present();
    let weak = text.downgrade();
    let manager = s.manager.clone();
    let names: std::collections::HashMap<_, _> = s
        .cameras
        .borrow()
        .iter()
        .map(|c| (c.id.clone(), c.name.clone()))
        .collect();
    let previous = Rc::new(std::cell::Cell::new(None::<(u64, u64)>));
    glib::timeout_add_local(Duration::from_secs(1), move || {
        if weak.upgrade().is_none() {
            return glib::ControlFlow::Break;
        }
        let manager = manager.clone();
        let weak = weak.clone();
        let names = names.clone();
        let prev = previous.get();
        let previous = previous.clone();
        jobs::run(
            move || {
                // Read small /proc counters on a worker. CPU is measured as share of total machine capacity.
                let process = std::fs::read_to_string("/proc/self/stat")
                    .ok()
                    .and_then(|line| {
                        let (_, fields) = line.rsplit_once(')')?;
                        let f: Vec<_> = fields.split_whitespace().collect();
                        Some(f.get(11)?.parse::<u64>().ok()? + f.get(12)?.parse::<u64>().ok()?)
                    })
                    .unwrap_or(0);
                let total = std::fs::read_to_string("/proc/stat")
                    .ok()
                    .and_then(|line| {
                        line.lines().next().map(|s| {
                            s.split_whitespace()
                                .skip(1)
                                .filter_map(|v| v.parse::<u64>().ok())
                                .sum()
                        })
                    })
                    .unwrap_or(0);

                let memory = std::fs::read_to_string("/proc/self/status")
                    .ok()
                    .and_then(|s| {
                        s.lines()
                            .find(|l| l.starts_with("VmRSS:"))
                            .map(String::from)
                    })
                    .unwrap_or_else(|| "Memória não disponível".into());
                let cpu = prev
                    .map(|(p, t)| {
                        100. * process.saturating_sub(p) as f64
                            / total.saturating_sub(t).max(1) as f64
                    })
                    .unwrap_or(0.);
                let mut output = format!("CPU: {cpu:.2}% da máquina · {memory}\n\n");
                if let Ok(m) = manager.lock() {
                    for stat in m.statistics() {
                        let name = names
                            .get(&stat.camera_id)
                            .map(String::as_str)
                            .unwrap_or("Câmera");
                        output.push_str(&format!("{name}\n  Decoder: {} · frames: {} · descartes: {}\n  Idade do último frame: {:?} ms · áudio entregue: {} bytes\n\n",if stat.hardware{"VA-API"}else{"software"},stat.frames,stat.dropped,stat.frame_age_ms,stat.audio_bytes));
                    }
                }
                output.push_str("Idade do frame mede o buffer local; não é latência câmera-tela.\nÁudio entregue é PCM aceito pelo servidor de áudio.");
                (output, process, total)
            },
            move |(output, process, total)| {
                previous.set(Some((process, total)));
                if let Some(text) = weak.upgrade() {
                    text.buffer().set_text(&output);
                }
            },
        );
        glib::ControlFlow::Continue
    });
}
