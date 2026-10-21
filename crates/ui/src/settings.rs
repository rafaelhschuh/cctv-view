use super::*;
pub(crate) fn open(s: &Shared) {
    let d = gtk::Dialog::builder()
        .transient_for(&s.window)
        .destroy_with_parent(true)
        .modal(true)
        .title("Preferências")
        .default_width(600)
        .default_height(520)
        .build();
    d.add_button("Fechar", gtk::ResponseType::Close);
    let notebook = gtk::Notebook::new();
    notebook.set_vexpand(true);
    d.content_area().append(&notebook);
    let general = forms::page(&notebook, "Geral");
    forms::section(&general, "Áudio");
    forms::help(&general, "Somente a câmera em foco reproduz áudio.");
    let volume = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0., 100., 1.);
    volume.set_value(s.preferences.borrow().volume * 100.);
    forms::row(&general, "Volume da câmera em foco", &volume);
    let state = s.clone();
    volume.connect_value_changed(move |v| {
        let value = v.value() / 100.;
        state.preferences.borrow_mut().volume = value;
        if let Ok(m) = state.manager.lock() {
            m.audio.set_volume(value);
        }
        save_preferences(&state);
    });
    forms::section(&general, "Aparência e navegação");
    forms::help(
        &general,
        "Tema e fontes acompanham as preferências do sistema.",
    );
    forms::help(&general,"F11: tela cheia\nEscape: voltar à grade ou sair da tela cheia\nDuplo clique: entrar ou sair do foco\nCtrl+Q: sair do aplicativo");
    forms::section(&general, "Suporte");
    let diagnostics = gtk::Button::with_label("Diagnóstico de sessões e recursos");
    diagnostics.set_halign(gtk::Align::Start);
    general.append(&diagnostics);
    let state = s.clone();
    diagnostics.connect_clicked(move |_| diagnostics::open(&state));

    let library = forms::page(&notebook, "Biblioteca");
    forms::section(&library, "Transferir câmeras");
    forms::help(&library,"Exporte ou importe câmeras e pastas em JSON criptografado, incluindo credenciais, URLs completas e configurações. A importação adiciona cadastros sem substituir os existentes.");
    let export = gtk::Button::with_label("Exportar câmeras…");
    export.set_halign(gtk::Align::Start);
    library.append(&export);
    let import = gtk::Button::with_label("Importar câmeras…");
    import.set_halign(gtk::Align::Start);
    library.append(&import);
    forms::section(&library, "Senha do arquivo");
    forms::help(&library,"Ao exportar, escolha uma senha para proteger o arquivo. Use a mesma senha para importar. Se o aplicativo tiver uma senha definida, ela também será solicitada antes de editar, clonar ou exportar câmeras.");
    let state = s.clone();
    let parent = d.downgrade();
    export.connect_clicked(move |_| {
        if let Some(parent) = parent.upgrade() {
            transfer::choose(&state, &parent, true);
        }
    });
    let state = s.clone();
    let parent = d.downgrade();
    import.connect_clicked(move |_| {
        if let Some(parent) = parent.upgrade() {
            transfer::choose(&state, &parent, false);
        }
    });

    let security = forms::page(&notebook, "Segurança");
    forms::section(&security, "Acesso ao aplicativo");
    forms::help(&security,"Defina, altere ou remova a senha de abertura. Alterar ou remover exige a senha atual. Esse bloqueio protege a interface; os arquivos locais não são criptografados.");
    let password = gtk::Button::with_label("Senha do aplicativo…");
    password.set_halign(gtk::Align::Start);
    security.append(&password);
    let state = s.clone();
    let parent = d.downgrade();
    password.connect_clicked(move |_| {
        if let Some(parent) = parent.upgrade() {
            auth::configure(&state, &parent);
        }
    });
    let lock = gtk::Button::with_label("Bloquear agora");
    lock.set_halign(gtk::Align::Start);
    security.append(&lock);
    forms::help(
        &security,
        "Interrompe a reprodução e solicita a senha configurada para voltar.",
    );
    let state = s.clone();
    lock.connect_clicked(move |_| auth::lock(&state));
    d.connect_response(|d, _| d.close());
    d.present();
}
