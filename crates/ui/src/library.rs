use super::*;
use std::{cell::Cell, rc::Weak};

struct Library {
    state: Shared,
    window: glib::WeakRef<gtk::Dialog>,
    list: gtk::ListBox,
    search: gtk::SearchEntry,
    folder: gtk::DropDown,
    folders: RefCell<Vec<String>>,
    count: gtk::Label,
    rename: gtk::Button,
    delete: gtk::Button,
    busy: Cell<bool>,
    revision: Cell<u64>,
    controls: gtk::Box,
    select_after_reload: RefCell<Option<String>>,
}
pub(crate) fn open(s: &Shared) {
    let d = gtk::Dialog::builder()
        .transient_for(&s.window)
        .destroy_with_parent(true)
        .title("Biblioteca de câmeras")
        .default_width(860)
        .default_height(580)
        .build();
    d.add_button("Fechar", gtk::ResponseType::Close);
    let area = d.content_area();
    margins(&area, 16);
    area.set_spacing(12);
    let top = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let search = gtk::SearchEntry::new();
    search.set_placeholder_text(Some("Buscar câmera por nome ou endereço"));
    search.set_hexpand(true);
    search.update_property(&[gtk::accessible::Property::Label("Buscar câmeras")]);
    let add = gtk::Button::with_label("Adicionar câmera");
    add.add_css_class("suggested-action");
    let discover = gtk::Button::with_label("Descobrir ONVIF");
    top.append(&search);
    top.append(&discover);
    top.append(&add);
    area.append(&top);
    let filters = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let folder = gtk::DropDown::from_strings(&["Todas as câmeras", "Sem pasta"]);
    folder.set_hexpand(true);
    folder.set_tooltip_text(Some("Filtrar por pasta"));
    folder.update_property(&[gtk::accessible::Property::Label("Filtrar por pasta")]);
    let create = gtk::Button::with_label("Nova pasta");
    let rename = button("Renomear pasta", "document-edit-symbolic");
    let delete = button("Excluir pasta", "user-trash-symbolic");
    filters.append(&folder);
    filters.append(&create);
    filters.append(&rename);
    filters.append(&delete);
    area.append(&filters);
    let count = gtk::Label::new(Some("Carregando biblioteca…"));
    count.set_xalign(0.);
    count.add_css_class("dim-label");
    area.append(&count);
    let list = gtk::ListBox::new();
    list.set_selection_mode(gtk::SelectionMode::None);
    list.add_css_class("boxed-list");
    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .child(&list)
        .build();
    scroll.set_has_frame(true);
    area.append(&scroll);
    let l = Rc::new(Library {
        state: s.clone(),
        window: d.downgrade(),
        list,
        search: search.clone(),
        folder: folder.clone(),
        folders: RefCell::new(vec![]),
        count,
        rename: rename.clone(),
        delete: delete.clone(),
        busy: Cell::new(false),
        revision: Cell::new(0),
        controls: area.clone(),
        select_after_reload: RefCell::new(None),
    });
    let weak = Rc::downgrade(&l);
    search.connect_search_changed(move |_| {
        if let Some(l) = weak.upgrade() {
            l.populate();
        }
    });
    let weak = Rc::downgrade(&l);
    folder.connect_selected_notify(move |_| {
        if let Some(l) = weak.upgrade() {
            l.populate();
        }
    });
    let weak = Rc::downgrade(&l);
    add.connect_clicked(move |_| {
        if let Some(l) = weak.upgrade() {
            let c = Camera {
                folder: l.selected_folder(),
                ..Default::default()
            };
            camera::form(&l.state, c, Some(l.callback()));
        }
    });
    let weak = Rc::downgrade(&l);
    discover.connect_clicked(move |_| {
        if let Some(l) = weak.upgrade() {
            camera::discovery(&l.state, l.callback());
        }
    });
    let weak = Rc::downgrade(&l);
    create.connect_clicked(move |_| {
        if let Some(l) = weak.upgrade() {
            l.folder_form(false);
        }
    });
    let weak = Rc::downgrade(&l);
    rename.connect_clicked(move |_| {
        if let Some(l) = weak.upgrade() {
            l.folder_form(true);
        }
    });
    let weak = Rc::downgrade(&l);
    delete.connect_clicked(move |_| {
        if let Some(l) = weak.upgrade() {
            if let Some(folder) = l.selected_folder() {
                let weak = Rc::downgrade(&l);
                confirm(
                    &l.state.window,
                    "Excluir a pasta? As câmeras serão mantidas em Sem pasta.",
                    move || {
                        if let Some(l) = weak.upgrade() {
                            l.mutate(move |store| store.change_folder(&folder, None));
                        }
                    },
                );
            }
        }
    });
    let weak = Rc::downgrade(&l);
    d.connect_is_active_notify(move |d| {
        if d.is_active() {
            if let Some(l) = weak.upgrade() {
                if !l.busy.get() {
                    l.reload();
                }
            }
        }
    });
    l.reload();
    // This signal owns the model until the window closes; row callbacks hold only Weak.
    d.connect_response(|d, _| forms::close(d));
    d.connect_close_request(move |_| {
        while let Some(c) = l.list.first_child() {
            l.list.remove(&c);
        }
        glib::Propagation::Proceed
    });
    d.present();
}
impl Library {
    fn selected_folder(&self) -> Option<String> {
        self.folder
            .selected()
            .checked_sub(2)
            .and_then(|i| self.folders.borrow().get(i as usize).cloned())
    }
    fn callback(self: &Rc<Self>) -> camera::LibraryRefresh {
        let weak = Rc::downgrade(self);
        Rc::new(move || {
            if let Some(l) = weak.upgrade() {
                l.reload();
            }
        })
    }
    fn reload(self: &Rc<Self>) {
        if self.busy.get() {
            return;
        }
        let revision = self.revision.get().wrapping_add(1);
        self.revision.set(revision);
        let path = self.state.path.clone();
        let weak = Rc::downgrade(self);
        jobs::run(
            move || -> Result<_> {
                let store = Store::open(path)?;
                Ok((store.cameras()?, store.folders()?))
            },
            move |result| Self::loaded(&weak, revision, result),
        );
    }
    fn loaded(weak: &Weak<Self>, revision: u64, result: Result<(Vec<Camera>, Vec<String>)>) {
        let Some(l) = weak.upgrade().filter(|l| l.window.upgrade().is_some()) else {
            return;
        };
        if l.revision.get() != revision {
            return;
        }
        l.busy.set(false);
        l.controls.set_sensitive(true);
        match result {
            Ok((cameras, folders)) => {
                let current = l
                    .select_after_reload
                    .borrow_mut()
                    .take()
                    .or_else(|| l.selected_folder());
                let index = l.folder.selected().min(1);
                *l.state.cameras.borrow_mut() = cameras;
                let mut names = vec!["Todas as câmeras".into(), "Sem pasta".into()];
                names.extend(folders.clone());
                let selected = current
                    .as_ref()
                    .and_then(|f| folders.iter().position(|n| n == f))
                    .map(|i| i as u32 + 2)
                    .unwrap_or(index);
                *l.folders.borrow_mut() = folders;
                l.folder.set_model(Some(&gtk::StringList::new(
                    &names.iter().map(String::as_str).collect::<Vec<_>>(),
                )));
                l.folder.set_selected(selected);
                l.populate();
            }
            Err(e) => message(&l.state.window, &e.to_string()),
        }
    }
    fn mutate(self: &Rc<Self>, work: impl FnOnce(&mut Store) -> Result<()> + Send + 'static) {
        if self.busy.replace(true) {
            return;
        }
        self.controls.set_sensitive(false);
        self.count.set_text("Salvando alterações…");
        let revision = self.revision.get().wrapping_add(1);
        self.revision.set(revision);
        let path = self.state.path.clone();
        let weak = Rc::downgrade(self);
        jobs::run(
            move || -> Result<_> {
                let mut store = Store::open(path)?;
                work(&mut store)?;
                Ok((store.cameras()?, store.folders()?))
            },
            move |result| Self::loaded(&weak, revision, result),
        );
    }
    fn populate(self: &Rc<Self>) {
        while let Some(c) = self.list.first_child() {
            self.list.remove(&c);
        }
        let folder = self.selected_folder();
        self.rename.set_sensitive(folder.is_some());
        self.delete.set_sensitive(folder.is_some());
        let query = self.search.text().to_lowercase();
        let cameras = self.state.cameras.borrow();
        let filtered: Vec<_> = cameras
            .iter()
            .filter(|c| {
                let matches = match self.folder.selected() {
                    0 => true,
                    1 => c.folder.is_none(),
                    _ => c.folder == folder,
                };
                matches
                    && (c.name.to_lowercase().contains(&query)
                        || c.address.to_lowercase().contains(&query)
                        || c.manufacturer.to_lowercase().contains(&query))
            })
            .collect();
        self.count
            .set_text(&format!("{} de {} câmeras", filtered.len(), cameras.len()));
        if filtered.is_empty() {
            let empty = gtk::Label::new(Some(if cameras.is_empty() {
                "Sua biblioteca está vazia. Adicione uma câmera ou descubra dispositivos ONVIF."
            } else {
                "Nenhuma câmera encontrada. Altere a busca ou a pasta."
            }));
            empty.set_wrap(true);
            margins(&empty, 24);
            self.list.append(&empty);
        }
        let type_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
        for c in filtered {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            margins(&row, 10);
            let icon = gtk::Image::from_icon_name("camera-video-symbolic");
            row.append(&icon);
            let info = gtk::Box::new(gtk::Orientation::Vertical, 4);
            info.set_hexpand(true);
            let name = gtk::Label::new(Some(&c.name));
            name.set_xalign(0.);
            name.set_ellipsize(gtk::pango::EllipsizeMode::End);
            name.add_css_class("heading");
            name.set_tooltip_text(Some(&c.name));
            info.append(&name);
            let subtitle = gtk::Label::new(Some(&format!(
                "{} · {}",
                c.folder.as_deref().unwrap_or("Sem pasta"),
                if c.address.is_empty() {
                    "Endereço não informado"
                } else {
                    &c.address
                }
            )));
            subtitle.set_xalign(0.);
            subtitle.set_ellipsize(gtk::pango::EllipsizeMode::End);
            subtitle.add_css_class("dim-label");
            info.append(&subtitle);
            row.append(&info);
            let kind = gtk::Label::new(Some(match c.stream_kind {
                cctv_core::StreamKind::Rtsp => "RTSP",
                cctv_core::StreamKind::Http => "HTTP(S)",
                cctv_core::StreamKind::Mjpeg => "MJPEG",
                cctv_core::StreamKind::Jpeg => "JPEG",
                _ => "Auto",
            }));
            kind.add_css_class("dim-label");
            type_group.add_widget(&kind);
            row.append(&kind);
            let edit = button("Editar câmera", "document-edit-symbolic");
            let clone = button("Clonar câmera sem senha", "edit-copy-symbolic");
            let move_to = button("Mover para pasta", "folder-symbolic");
            let delete = button("Excluir câmera", "user-trash-symbolic");
            for b in [&edit, &clone, &move_to, &delete] {
                b.set_valign(gtk::Align::Center);
                b.set_has_frame(false);
                row.append(b);
            }
            self.list.append(&row);
            let weak = Rc::downgrade(self);
            let c2 = c.clone();
            edit.connect_clicked(move |_| {
                if let Some(l) = weak.upgrade() {
                    camera::form(&l.state, c2.clone(), Some(l.callback()));
                }
            });
            let weak = Rc::downgrade(self);
            let id = c.id.clone();
            clone.connect_clicked(move |_| {
                if let Some(l) = weak.upgrade() {
                    l.clone_form(&id);
                }
            });
            let weak = Rc::downgrade(self);
            let c2 = c.clone();
            move_to.connect_clicked(move |_| {
                if let Some(l) = weak.upgrade() {
                    l.move_form(&c2);
                }
            });
            let weak = Rc::downgrade(self);
            let c = c.clone();
            delete.connect_clicked(move |_| {
                if let Some(l) = weak.upgrade() {
                    l.delete_camera(&c);
                }
            });
        }
    }
    fn folder_form(self: &Rc<Self>, rename: bool) {
        let old = if rename { self.selected_folder() } else { None };
        if rename && old.is_none() {
            return;
        }
        let d = gtk::Dialog::builder()
            .transient_for(&self.state.window)
            .modal(true)
            .destroy_with_parent(true)
            .title(if rename {
                "Renomear pasta"
            } else {
                "Nova pasta"
            })
            .default_width(380)
            .build();
        d.add_button("Cancelar", gtk::ResponseType::Cancel);
        d.add_button("Salvar", gtk::ResponseType::Accept);
        d.set_default_response(gtk::ResponseType::Accept);
        let area = d.content_area();
        margins(&area, 16);
        area.set_spacing(12);
        let name = forms::entry(&area, "Nome da pasta", old.as_deref().unwrap_or(""));
        let error = forms::feedback(&area);
        let weak = Rc::downgrade(self);
        d.connect_response(move |d, response| {
            if response != gtk::ResponseType::Accept {
                forms::close(d);
                return;
            }
            let Some(l) = weak.upgrade() else {
                return;
            };
            let name = name.text().trim().to_string();
            if name.is_empty() || name.len() > 160 || name.chars().any(char::is_control) {
                error.set_text("Informe um nome de até 160 bytes, sem caracteres de controle.");
                return;
            }
            if l.folders
                .borrow()
                .iter()
                .any(|f| Some(f) != old.as_ref() && f.to_lowercase() == name.to_lowercase())
            {
                error.set_text("Já existe uma pasta com esse nome.");
                return;
            }
            if old.is_some() {
                *l.select_after_reload.borrow_mut() = Some(name.clone());
            }
            let old = old.clone();
            l.mutate(move |store| match old {
                Some(old) => store.change_folder(&old, Some(&name)),
                None => store.create_folder(&name),
            });
            forms::close(d);
        });
        d.present();
    }
    fn move_form(self: &Rc<Self>, camera: &Camera) {
        let d = gtk::Dialog::builder()
            .transient_for(&self.state.window)
            .modal(true)
            .destroy_with_parent(true)
            .title("Mover câmera")
            .default_width(400)
            .build();
        d.add_button("Cancelar", gtk::ResponseType::Cancel);
        d.add_button("Mover", gtk::ResponseType::Accept);
        let area = d.content_area();
        margins(&area, 16);
        area.set_spacing(12);
        let folders = self.folders.borrow().clone();
        let mut names = vec!["Sem pasta".into()];
        names.extend(folders.clone());
        let choice =
            gtk::DropDown::from_strings(&names.iter().map(String::as_str).collect::<Vec<_>>());
        choice.set_selected(
            camera
                .folder
                .as_ref()
                .and_then(|f| folders.iter().position(|n| n == f))
                .map(|i| i as u32 + 1)
                .unwrap_or(0),
        );
        forms::row(&area, "Pasta de destino", &choice);
        let weak = Rc::downgrade(self);
        let id = camera.id.clone();
        d.connect_response(move |d, r| {
            if r == gtk::ResponseType::Accept {
                if let Some(l) = weak.upgrade() {
                    let folder = choice
                        .selected()
                        .checked_sub(1)
                        .and_then(|i| folders.get(i as usize).cloned());
                    let id = id.clone();
                    l.mutate(move |store| store.move_camera(&id, folder.as_deref()));
                }
            }
            forms::close(d);
        });
        d.present();
    }
    fn clone_form(self: &Rc<Self>, id: &str) {
        if self.busy.get() {
            return;
        }
        let id = id.to_string();
        let path = self.state.path.clone();
        let weak = Rc::downgrade(self);
        jobs::run(
            move || Store::open(path)?.prepare_camera_clone(&id),
            move |result| {
                if let Some(l) = weak.upgrade() {
                    match result {
                        Ok(camera) => camera::clone_form(&l.state, camera, Some(l.callback())),
                        Err(e) => message(&l.state.window, &e.to_string()),
                    }
                }
            },
        );
    }
    fn delete_camera(self: &Rc<Self>, c: &Camera) {
        let weak = Rc::downgrade(self);
        let c = c.clone();
        confirm(
            &self.state.window,
            "Excluir a câmera e removê-la de todas as Views?",
            move || {
                if let Some(l) = weak.upgrade() {
                    let path = l.state.path.clone();
                    let weak = Rc::downgrade(&l);
                    jobs::run(
                        move || -> Result<_> {
                            if let Some(secret) = c.secret_ref {
                                Vault::delete(&secret)?;
                            }
                            let mut store = Store::open(path)?;
                            store.delete_camera(&c.id)?;
                            Ok((store.cameras()?, store.views()?, store.folders()?))
                        },
                        move |r| {
                            if let Some(l) = weak.upgrade() {
                                match r {
                                    Ok((c, v, f)) => {
                                        *l.state.views.borrow_mut() = v;
                                        Self::loaded(&weak, l.revision.get(), Ok((c, f)));
                                        refresh(&l.state);
                                    }
                                    Err(e) => message(&l.state.window, &e.to_string()),
                                }
                            }
                        },
                    );
                }
            },
        );
    }
}
