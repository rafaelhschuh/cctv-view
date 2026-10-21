use super::*;
use cctv_storage::{CameraArchive, MAX_ARCHIVE_BYTES};
use std::io::{Read, Write};
pub(crate) fn choose(s: &Shared, parent: &gtk::Dialog, export: bool) {
    if export {
        let state = s.clone();
        let weak = parent.downgrade();
        auth::confirm_action(s, parent, "Exportar câmeras e credenciais", move || {
            if let Some(parent) = weak.upgrade().filter(|d| d.is_visible()) {
                password_dialog(&state, &parent, None);
            }
        });
    } else {
        file_picker(s, parent, None);
    }
}
fn file_picker(s: &Shared, parent: &gtk::Dialog, export_data: Option<String>) {
    let export = export_data.is_some();
    let chooser = gtk::FileChooserNative::new(
        Some(if export {
            "Exportar câmeras criptografadas"
        } else {
            "Importar câmeras criptografadas"
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
    filter.set_name(Some("Câmeras criptografadas (*.json)"));
    filter.add_pattern("*.json");
    chooser.add_filter(&filter);
    if export {
        chooser.set_current_name("cctv-cameras.json");
    }
    let state = s.clone();
    let parent = parent.downgrade();
    chooser.connect_response(move |chooser, response| {
        if response == gtk::ResponseType::Accept
            && !state.locked.get()
            && parent.upgrade().is_some_and(|d| d.is_visible())
        {
            if let Some(path) = chooser.file().and_then(|f| f.path()) {
                if let Some(data) = export_data.clone() {
                    save_file(&state, &parent, path, data);
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
                    "Câmeras exportadas em arquivo criptografado.",
                ),
                Err(e) => message(&state.window, &e.to_string()),
            }
        },
    );
}

pub(crate) fn password_dialog(s: &Shared, parent: &gtk::Dialog, encrypted: Option<Vec<u8>>) {
    let export = encrypted.is_none();
    let d = gtk::Dialog::builder()
        .transient_for(parent)
        .destroy_with_parent(true)
        .modal(true)
        .default_width(460)
        .title(if export {
            "Senha da exportação"
        } else {
            "Senha do arquivo"
        })
        .build();
    d.add_button("Cancelar", gtk::ResponseType::Cancel);
    d.add_button(
        if export {
            "Escolher arquivo…"
        } else {
            "Desbloquear arquivo"
        },
        gtk::ResponseType::Accept,
    )
    .add_css_class("suggested-action");
    d.set_default_response(gtk::ResponseType::Accept);
    let area = d.content_area();
    margins(&area, 20);
    area.set_spacing(12);
    forms::help(
        &area,
        if export {
            "Defina a senha para criptografar este arquivo. Câmeras, credenciais, URLs e configurações serão incluídas. Essa senha será solicitada ao importar."
        } else {
            "Informe a senha escolhida ao exportar este arquivo."
        },
    );
    let password = gtk::PasswordEntry::builder()
        .show_peek_icon(true)
        .activates_default(true)
        .build();
    password.set_tooltip_text(Some("Senha do arquivo"));
    forms::row(&area, "Senha do arquivo", &password);
    let confirmation = if export {
        let field = gtk::PasswordEntry::builder()
            .show_peek_icon(true)
            .activates_default(true)
            .build();
        field.set_tooltip_text(Some("Confirmar senha do arquivo"));
        forms::row(&area, "Confirmar senha do arquivo", &field);
        Some(field)
    } else {
        None
    };
    let feedback = forms::feedback(&area);
    let state = s.clone();
    let parent = parent.downgrade();
    let busy = Rc::new(std::cell::Cell::new(false));
    d.connect_response(move |d, response| {
        if response != gtk::ResponseType::Accept {
            password.set_text("");
            if let Some(c) = &confirmation {
                c.set_text("");
            }
            forms::close(d);
            return;
        }
        if busy.get() {
            return;
        }
        let candidate = password.text().to_string();
        if candidate.is_empty() || candidate.len() > 1024 {
            feedback.set_text("Informe uma senha de até 1024 bytes.");
            return;
        }
        if confirmation.as_ref().is_some_and(|c| c.text() != candidate) {
            feedback.set_text("As senhas não coincidem.");
            return;
        }
        busy.set(true);
        d.set_response_sensitive(gtk::ResponseType::Accept, false);
        feedback.set_text(if export {
            "Criptografando…"
        } else {
            "Descriptografando…"
        });
        password.set_text("");
        if let Some(c) = &confirmation {
            c.set_text("");
        }
        let database = state.path.clone();
        let data = encrypted.clone();
        let state = state.clone();
        let parent = parent.clone();
        let weak = d.downgrade();
        let feedback = feedback.clone();
        let busy = busy.clone();
        jobs::run(
            move || -> Result<_> {
                if let Some(data) = data {
                    Ok(TransferResult::Import(CameraArchive::decrypt(
                        &data, &candidate,
                    )?))
                } else {
                    Ok(TransferResult::Export(
                        Store::open(database)?.export_encrypted(&candidate)?,
                    ))
                }
            },
            move |result| {
                busy.set(false);
                let Some(d) = weak
                    .upgrade()
                    .filter(|d| d.is_visible() && !state.locked.get())
                else {
                    return;
                };
                d.set_response_sensitive(gtk::ResponseType::Accept, true);
                match result {
                    Ok(result) => {
                        forms::close(&d);
                        if let Some(parent) = parent.upgrade().filter(|d| d.is_visible()) {
                            match result {
                                TransferResult::Export(data) => {
                                    file_picker(&state, &parent, Some(data))
                                }
                                TransferResult::Import(archive) => {
                                    preview(&state, &parent, archive)
                                }
                            }
                        }
                    }
                    Err(e) => feedback.set_text(&e.to_string()),
                }
            },
        );
    });
    d.present();
}
enum TransferResult {
    Export(String),
    Import(CameraArchive),
}
pub(super) fn read_file(s: &Shared, parent: &glib::WeakRef<gtk::Dialog>, path: PathBuf) {
    let state = s.clone();
    let parent = parent.clone();
    jobs::run(
        move || -> Result<Vec<u8>> {
            let file = std::fs::File::open(path)
                .map_err(|_| anyhow::anyhow!("Não foi possível abrir o arquivo selecionado."))?;
            let mut data = vec![];
            file.take(MAX_ARCHIVE_BYTES + 1).read_to_end(&mut data)?;
            anyhow::ensure!(
                data.len() as u64 <= MAX_ARCHIVE_BYTES,
                "O arquivo excede 12 MiB."
            );
            Ok(data)
        },
        move |result| {
            let Some(parent) = parent
                .upgrade()
                .filter(|d| d.is_visible() && !state.locked.get())
            else {
                return;
            };
            match result {
                Ok(data) => password_dialog(&state, &parent, Some(data)),
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
    forms::help(&area,"Serão adicionadas novas câmeras com suas credenciais e configurações, sem substituir os cadastros existentes.");
    let list = gtk::Box::new(gtk::Orientation::Vertical, 8);
    for c in &archive.cameras {
        let name = gtk::Label::new(Some(&format!(
            "{} · {}",
            c.camera.name,
            c.camera.folder.as_deref().unwrap_or("Sem pasta")
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
                        message(
                            &state.window,
                            &format!(
                                "{count} câmeras importadas com suas configurações e credenciais."
                            ),
                        );
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
