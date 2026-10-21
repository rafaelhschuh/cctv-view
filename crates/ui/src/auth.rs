use super::*;

pub(crate) fn gate(
    window: &gtk::ApplicationWindow,
    path: PathBuf,
    on_unlock: impl FnOnce() + 'static,
) {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 16);
    content.set_valign(gtk::Align::Center);
    content.set_halign(gtk::Align::Center);
    content.set_width_request(320);
    margins(&content, 24);
    let icon = gtk::Image::from_icon_name("system-lock-screen-symbolic");
    icon.set_pixel_size(48);
    content.append(&icon);
    forms::section(&content, "CCTV View bloqueado");
    let password = gtk::PasswordEntry::builder().show_peek_icon(true).build();
    password.set_tooltip_text(Some("Senha do aplicativo"));
    forms::row(&content, "Senha do aplicativo", &password);
    let feedback = gtk::Label::new(None);
    feedback.set_wrap(true);
    content.append(&feedback);
    let unlock = gtk::Button::with_label("Desbloquear");
    unlock.add_css_class("suggested-action");
    content.append(&unlock);
    let close = gtk::Button::with_label("Fechar aplicativo");
    content.append(&close);
    let weak = window.downgrade();
    close.connect_clicked(move |_| {
        if let Some(w) = weak.upgrade() {
            w.close();
        }
    });
    let callback = Rc::new(RefCell::new(Some(on_unlock)));
    let busy = Rc::new(std::cell::Cell::new(false));
    let button = unlock.downgrade();
    password.connect_activate(move |_| {
        if let Some(b) = button.upgrade() {
            b.emit_clicked();
        }
    });
    let field = password.clone();
    let alive = window.downgrade();
    unlock.connect_clicked(move |button| {
        if busy.replace(true) {
            return;
        }
        button.set_sensitive(false);
        let candidate = field.text().to_string();
        field.set_text("");
        feedback.set_text("Verificando…");
        let path = path.clone();
        let callback = callback.clone();
        let busy = busy.clone();
        let field = field.clone();
        let feedback = feedback.clone();
        let button = button.downgrade();
        let alive = alive.clone();
        jobs::run(
            move || Store::open(path)?.verify_app_password(&candidate),
            move |result: Result<bool>| {
                if !alive.upgrade().is_some_and(|w| w.is_visible()) {
                    callback.borrow_mut().take();
                    return;
                }
                match result {
                    Ok(true) => {
                        if let Some(callback) = callback.borrow_mut().take() {
                            callback();
                        }
                    }
                    result => {
                        feedback.set_text(&match result {
                            Ok(false) => "Senha incorreta.".into(),
                            Err(e) => e.to_string(),
                            _ => String::new(),
                        });
                        glib::timeout_add_local_once(Duration::from_millis(750), move || {
                            busy.set(false);
                            if let Some(b) = button.upgrade() {
                                b.set_sensitive(true);
                                field.grab_focus();
                            }
                        });
                    }
                }
            },
        );
    });
    window.set_child(Some(&content));
    password.grab_focus();
}

pub(crate) fn configure(s: &Shared, parent: &impl IsA<gtk::Window>) {
    let dialog = gtk::Dialog::builder()
        .title("Senha do aplicativo")
        .transient_for(parent)
        .destroy_with_parent(true)
        .modal(true)
        .default_width(440)
        .build();
    dialog.add_button("Cancelar", gtk::ResponseType::Cancel);
    dialog
        .add_button("Salvar senha", gtk::ResponseType::Accept)
        .add_css_class("suggested-action");
    let area = dialog.content_area();
    margins(&area, 20);
    forms::help(&area, "Defina uma senha para abrir ou desbloquear o aplicativo. Para alterar ou remover uma senha existente, informe a senha atual.");
    let mut fields = vec![];
    for label in ["Senha atual", "Nova senha", "Confirmar nova senha"] {
        let field = gtk::PasswordEntry::builder().show_peek_icon(true).build();
        field.set_tooltip_text(Some(label));
        forms::row(&area, label, &field);
        fields.push(field);
    }
    let remove = gtk::CheckButton::with_label("Remover a senha do aplicativo");
    area.append(&remove);
    let error = gtk::Label::new(None);
    error.set_wrap(true);
    area.append(&error);
    let path = s.path.clone();
    let busy = Rc::new(std::cell::Cell::new(false));
    dialog.connect_response(move |d, response| {
        if response != gtk::ResponseType::Accept {
            d.close();
            return;
        }
        if busy.replace(true) {
            return;
        }
        let current = fields[0].text().to_string();
        let next = fields[1].text().to_string();
        if !remove.is_active() && next != fields[2].text() {
            busy.set(false);
            error.set_text("As senhas não coincidem.");
            return;
        }
        if !remove.is_active() && (next.chars().count() < 8 || next.len() > 1024) {
            busy.set(false);
            error.set_text("Use pelo menos 8 caracteres (até 1024 bytes).");
            return;
        }
        let next = if remove.is_active() { None } else { Some(next) };
        for field in &fields {
            field.set_text("");
        }
        d.set_response_sensitive(gtk::ResponseType::Accept, false);
        let weak = d.downgrade();
        let error = error.clone();
        let path = path.clone();
        let busy = busy.clone();
        jobs::run(
            move || Store::open(path)?.set_app_password(&current, next.as_deref()),
            move |result| {
                busy.set(false);
                if let Some(d) = weak.upgrade() {
                    match result {
                        Ok(()) => d.close(),
                        Err(e) => {
                            error.set_text(&e.to_string());
                            d.set_response_sensitive(gtk::ResponseType::Accept, true);
                        }
                    }
                }
            },
        );
    });
    dialog.present();
}

pub(crate) fn lock(s: &Shared) {
    let path = s.path.clone();
    let state = Rc::downgrade(s);
    jobs::run(
        move || Store::open(path)?.has_app_password(),
        move |result| {
            let Some(s) = state.upgrade() else { return };
            if !s.window.is_visible() {
                return;
            }
            match result {
                Ok(false) => configure(&s, &s.window),
                Err(e) => message(&s.window, &e.to_string()),
                Ok(true) => {
                    if s.locked.replace(true) {
                        return;
                    }
                    if let Some(app) = s.window.application() {
                        for w in app.windows() {
                            if w != s.window {
                                w.close();
                            }
                        }
                    }
                    if let Ok(mut manager) = s.manager.lock() {
                        manager.shutdown();
                    }
                    while let Some(child) = s.grid.first_child() {
                        s.grid.remove(&child);
                    }
                    s.presentation.set(false);
                    s.window.unfullscreen();
                    s.window.set_decorated(true);
                    if let Some(header) = s.header.upgrade() {
                        header.set_visible(false);
                    }
                    let Some(root) = s.window.child() else { return };
                    let weak = Rc::downgrade(&s);
                    gate(&s.window, s.path.clone(), move || {
                        if let Some(s) = weak.upgrade() {
                            s.locked.set(false);
                            s.window.set_child(Some(&root));
                            render(&s);
                        }
                    });
                }
            }
        },
    );
}
