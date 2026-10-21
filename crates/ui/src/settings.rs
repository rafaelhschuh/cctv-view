use super::*;
pub(crate) fn open(s: &Shared) {
    let d = gtk::Dialog::builder()
        .transient_for(&s.window)
        .destroy_with_parent(true)
        .modal(true)
        .title("Preferências")
        .default_width(420)
        .build();
    d.add_button("Fechar", gtk::ResponseType::Close);
    let area = d.content_area();
    margins(&area, 20);

    let volume = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0., 100., 1.);
    volume.set_value(s.preferences.borrow().volume * 100.);
    forms::row(&area, "Volume da câmera em foco", &volume);
    let s2 = s.clone();
    let diagnostics = gtk::Button::with_label("Diagnóstico de sessões e recursos");
    area.append(&diagnostics);
    diagnostics.connect_clicked(move |_| diagnostics::open(&s2));
    let s = s.clone();
    volume.connect_value_changed(move |v| {
        let value = v.value() / 100.;
        s.preferences.borrow_mut().volume = value;
        if let Ok(m) = s.manager.lock() {
            m.audio.set_volume(value);
        }
        save_preferences(&s);
    });
    let label=gtk::Label::new(Some("Tema e fontes seguem o sistema.\nÁudio: uma câmera por vez.\nF11: tela cheia · Escape: voltar · Ctrl+Q: sair"));
    label.set_wrap(true);
    area.append(&label);
    d.connect_response(|d, _| d.close());
    d.present();
}
