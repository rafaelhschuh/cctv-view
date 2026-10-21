use super::*;
use cctv_storage::{CameraArchive, MAX_ARCHIVE_BYTES};
use std::io::{Read, Write};

pub(crate) fn choose(s: &Shared, parent: &gtk::Dialog, export: bool) {
    if export {
        prepare_export(s, parent);
        return;
    }
    file_picker(s, parent, None);
}
fn file_picker(s: &Shared, parent: &gtk::Dialog, export_data: Option<String>) {
    let export = export_data.is_some();
    let chooser = gtk::FileChooserNative::new(
        Some(if export {
            "Exportar câmeras sem senha"
        } else {
            "Importar câmeras sem senha"
        }),
        Some(parent),
        if export {
            gtk::FileChooserAction::Save
        } else {
            gtk::FileChooserAction::Open
        },
        Some(if export { "Exportar" } else { "Abrir" }),
        Some("Cancelar"),
    );
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("Câmeras CCTV View (*.json)"));
    filter.add_pattern("*.json");
    chooser.add_filter(&filter);
    if export {
        chooser.set_current_name("cctv-cameras.json");
    }
    let state = s.clone();
    let parent = parent.downgrade();
    chooser.connect_response(move |chooser, response| {
        if !state.locked.get()
            && parent.upgrade().is_some_and(|d| d.is_visible())
            && response == gtk::ResponseType::Accept
        {
            if let Some(path) = chooser.file().and_then(|f| f.path()) {
                if export {
                    if let Some(data) = export_data.clone() {
                        save_file(&state, &parent, path, data);
                    }
                } else {
                    read_file(&state, &parent, path);
                }
            } else {
                message(&state.window, "Selecione um arquivo local.");
            }
        }
        chooser.destroy();
    });
    chooser.show();
}
fn write_archive(path: &std::path::Path, data: &str) -> Result<()> {
    // Never truncate a previous export, including when a chooser backend skips confirmation.
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            anyhow::anyhow!("Esse arquivo já existe. Escolha outro nome para exportar.")
        } else {
            anyhow::anyhow!("Não foi possível criar o arquivo de exportação: {e}")
        }
    })?;
    if let Err(error) = file
        .write_all(data.as_bytes())
        .and_then(|_| file.sync_all())
    {
        drop(file);
        let _ = std::fs::remove_file(path);
        return Err(anyhow::anyhow!("Falha ao gravar o arquivo: {error}"));
    }
    Ok(())
}
pub(super) fn save_file(
    s: &Shared,
    parent: &glib::WeakRef<gtk::Dialog>,
    path: PathBuf,
    data: String,
) {
    let state = s.clone();
    let parent = parent.clone();
    jobs::run(
        move || write_archive(&path, &data),
        move |r| {
            if state.locked.get() || !parent.upgrade().is_some_and(|d| d.is_visible()) {
                return;
            }
            match r {
                Ok(()) => message(
                    &state.window,
                    "Câmeras exportadas sem credenciais do chaveiro.",
                ),
                Err(e) => message(&state.window, &e.to_string()),
            }
        },
    );
}
fn prepare_export(s: &Shared, parent: &gtk::Dialog) {
    let database = s.path.clone();
    let state = s.clone();
    let parent = parent.downgrade();
    jobs::run(
        move || -> Result<_> {
            CameraArchive::parse(Store::open(database)?.export_cameras()?.as_bytes())
        },
        move |r| {
            let Some(parent) = parent
                .upgrade()
                .filter(|d| d.is_visible() && !state.locked.get())
            else {
                return;
            };
            match r {
                Ok(archive) => export_review(&state, &parent, archive),
                Err(e) => message(&state.window, &e.to_string()),
            }
        },
    );
}
pub(crate) fn export_review(s: &Shared, parent: &gtk::Dialog, archive: CameraArchive) {
    let d = gtk::Dialog::builder()
        .transient_for(parent)
        .destroy_with_parent(true)
        .modal(true)
        .title("Revisar exportação")
        .default_width(640)
        .default_height(520)
        .build();
    d.add_button("Cancelar", gtk::ResponseType::Cancel);
    let accept = d.add_button("Escolher arquivo…", gtk::ResponseType::Accept);
    accept.add_css_class("suggested-action");
    accept.set_sensitive(false);
    let area = d.content_area();
    margins(&area, 16);
    area.set_spacing(12);
    forms::help(&area,"Credenciais do chaveiro não são incluídas. Parâmetros e fragmentos foram removidos. Revise e edite as URLs abaixo: uma senha ou um token também pode estar no caminho do endereço. As alterações valem somente para este arquivo.");
    let list = gtk::Box::new(gtk::Orientation::Vertical, 16);
    let mut entries = vec![];
    for camera in &archive.cameras {
        let group = gtk::Box::new(gtk::Orientation::Vertical, 8);
        forms::section(&group, &camera.name);
        let stream = forms::entry(&group, "URL do stream para exportar", &camera.stream_url);
        let substream = camera
            .substream_url
            .as_ref()
            .map(|value| forms::entry(&group, "URL secundária para exportar", value));
        let onvif = if camera.onvif_url.is_empty() {
            None
        } else {
            Some(forms::entry(
                &group,
                "URL ONVIF para exportar",
                &camera.onvif_url,
            ))
        };
        entries.push((stream, substream, onvif));
        list.append(&group);
    }
    let scroll = gtk::ScrolledWindow::builder()
        .child(&list)
        .vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();
    area.append(&scroll);
    let reviewed =
        gtk::CheckButton::with_label("Revisei as URLs e removi senhas ou tokens dos caminhos");
    area.append(&reviewed);
    let accept2 = accept.clone();
    reviewed.connect_toggled(move |b| accept2.set_sensitive(b.is_active()));
    for (stream, substream, onvif) in &entries {
        for entry in std::iter::once(stream)
            .chain(substream.iter())
            .chain(onvif.iter())
        {
            let reviewed = reviewed.clone();
            entry.connect_changed(move |_| reviewed.set_active(false));
        }
    }
    let feedback = forms::feedback(&area);
    let state = s.clone();
    let parent = parent.downgrade();
    let busy = Rc::new(std::cell::Cell::new(false));
    let busy_close = busy.clone();
    let close_state = Rc::downgrade(s);
    d.connect_close_request(move |_| {
        if busy_close.get() && close_state.upgrade().is_some_and(|s| !s.locked.get()) {
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    d.connect_response(move |d, r| {
        if busy.get() {
            return;
        }
        if r != gtk::ResponseType::Accept {
            forms::close(d);
            return;
        }
        if !reviewed.is_active() {
            return;
        }
        let mut archive = archive.clone();
        for (c, (stream, substream, onvif)) in archive.cameras.iter_mut().zip(&entries) {
            c.stream_url = stream.text().to_string();
            c.substream_url = substream.as_ref().map(|e| e.text().to_string());
            c.onvif_url = onvif
                .as_ref()
                .map(|e| e.text().to_string())
                .unwrap_or_default();
        }
        busy.set(true);
        accept.set_sensitive(false);
        let weak = d.downgrade();
        let state = state.clone();
        let parent = parent.clone();
        let feedback = feedback.clone();
        let accept = accept.clone();
        let busy = busy.clone();
        jobs::run(
            move || -> Result<_> {
                let data = serde_json::to_vec(&archive)?;
                let checked = CameraArchive::parse(&data)?;
                checked.sanitized_json()
            },
            move |r| {
                busy.set(false);
                let Some(d) = weak.upgrade() else {
                    return;
                };
                match r {
                    Ok(data) => {
                        forms::close(&d);
                        if let Some(parent) = parent.upgrade() {
                            file_picker(&state, &parent, Some(data));
                        }
                    }
                    Err(e) => {
                        accept.set_sensitive(true);
                        feedback.set_text(&e.to_string());
                    }
                }
            },
        );
    });
    d.present();
}
pub(super) fn read_file(s: &Shared, parent: &glib::WeakRef<gtk::Dialog>, path: PathBuf) {
    let state = s.clone();
    let parent = parent.clone();
    jobs::run(
        move || -> Result<_> {
            let file = std::fs::File::open(path)
                .map_err(|_| anyhow::anyhow!("Não foi possível abrir o arquivo selecionado."))?;
            let mut data = Vec::new();
            file.take(MAX_ARCHIVE_BYTES + 1).read_to_end(&mut data)?;
            CameraArchive::parse(&data)
        },
        move |r| {
            let Some(parent) = parent
                .upgrade()
                .filter(|d| d.is_visible() && !state.locked.get())
            else {
                return;
            };
            match r {
                Ok(archive) => preview(&state, &parent, archive),
                Err(e) => message(&state.window, &e.to_string()),
            }
        },
    );
}
pub(crate) fn preview(s: &Shared, parent: &gtk::Dialog, archive: CameraArchive) {
    let d = gtk::Dialog::builder()
        .transient_for(parent)
        .destroy_with_parent(true)
        .modal(true)
        .title("Revisar importação")
        .default_width(560)
        .default_height(440)
        .build();
    d.add_button("Cancelar", gtk::ResponseType::Cancel);
    let accept = d.add_button("Importar", gtk::ResponseType::Accept);
    accept.add_css_class("suggested-action");
    let area = d.content_area();
    margins(&area, 16);
    area.set_spacing(12);
    let summary = gtk::Label::new(Some(&format!(
        "{} câmeras · {} pastas",
        archive.cameras.len(),
        archive.folders.len()
    )));
    summary.set_xalign(0.);
    summary.add_css_class("heading");
    area.append(&summary);
    forms::help(&area,"Serão adicionadas novas câmeras, sem substituir cadastros existentes. Credenciais devem ser informadas novamente. Parâmetros e fragmentos das URLs não são transferidos; revise a conexão e consulte novamente o ONVIF se necessário.");
    let list = gtk::Box::new(gtk::Orientation::Vertical, 8);
    for c in &archive.cameras {
        let name = gtk::Label::new(Some(&format!(
            "{} · {}",
            c.name,
            c.folder.as_deref().unwrap_or("Sem pasta")
        )));
        name.set_xalign(0.);
        name.set_ellipsize(gtk::pango::EllipsizeMode::End);
        list.append(&name);
    }
    let scroll = gtk::ScrolledWindow::builder()
        .child(&list)
        .vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();
    area.append(&scroll);
    let feedback = forms::feedback(&area);
    let state = s.clone();
    let archive = Rc::new(RefCell::new(Some(archive)));
    let busy = Rc::new(std::cell::Cell::new(false));
    let busy_close = busy.clone();
    let close_state = Rc::downgrade(s);
    d.connect_close_request(move |_| {
        if busy_close.get() && close_state.upgrade().is_some_and(|s| !s.locked.get()) {
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    d.connect_response(move |d, response| {
        if busy.get() {
            return;
        }
        if response != gtk::ResponseType::Accept {
            forms::close(d);
            return;
        }
        let Some(archive) = archive.borrow_mut().take() else {
            return;
        };
        busy.set(true);
        if let Some(cancel) = d.widget_for_response(gtk::ResponseType::Cancel) {
            cancel.set_sensitive(false);
        }
        accept.set_sensitive(false);
        let path = state.path.clone();
        let state = state.clone();
        let weak = d.downgrade();
        let feedback = feedback.clone();
        let accept = accept.clone();
        let busy = busy.clone();
        jobs::run(
            move || -> Result<_> {
                let mut store = Store::open(path)?;
                let count = store.import_cameras(archive)?;
                Ok((count, store.cameras()?))
            },
            move |result| {
                busy.set(false);
                if let Ok((_, cameras)) = &result {
                    *state.cameras.borrow_mut() = cameras.clone();
                    refresh(&state);
                }
                if state.locked.get() {
                    return;
                }
                let Some(d) = weak.upgrade().filter(|d| d.is_visible()) else {
                    return;
                };
                if let Some(cancel) = d.widget_for_response(gtk::ResponseType::Cancel) {
                    cancel.set_sensitive(true);
                }
                match result {
                    Ok((count, _)) => {
                        forms::close(&d);
                        message(&state.window, &format!("{count} câmeras importadas sem credenciais do chaveiro. Revise as conexões na Biblioteca de câmeras."));
                    }
                    Err(error) => {
                        feedback.set_text(&error.to_string());
                        accept.set_visible(false);
                    }
                }
            },
        );
    });
    d.present();
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn export_does_not_overwrite_and_creates_private_file() -> Result<()> {
        let path = std::env::temp_dir().join(format!("cctv-export-{}.json", cctv_core::id()));
        write_archive(&path, "first")?;
        assert!(write_archive(&path, "second").is_err());
        assert_eq!(std::fs::read_to_string(&path)?, "first");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path)?.permissions().mode() & 0o777,
                0o600
            );
        }
        std::fs::remove_file(path)?;
        Ok(())
    }
}
