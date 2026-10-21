mod camera;
mod diagnostics;
mod editor;
mod jobs;
mod monitor;
mod player;
mod settings;
pub(crate) use monitor::{render, tile};
mod ptz;
use anyhow::Result;
use cctv_core::{Camera, Credentials, Preferences, View, APP_ID};
use cctv_media::{SessionManager, Status};
use cctv_storage::{Store, Vault};
use gtk::{gdk, gio, glib, prelude::*};
use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};

type Shared = Rc<State>;
pub(crate) struct State {
    window: gtk::ApplicationWindow,
    cameras: RefCell<Vec<Camera>>,
    views: RefCell<Vec<View>>,
    preferences: RefCell<Preferences>,
    current: RefCell<Option<String>>,
    focus: RefCell<Option<String>>,
    sidebar: gtk::Box,
    view_list: gtk::ListBox,
    grid: gtk::Grid,
    heading: gtk::Label,
    status: gtk::Label,
    path: PathBuf,
    manager: Arc<Mutex<SessionManager>>,
    runtime: Arc<tokio::runtime::Runtime>,
}
pub fn run() {
    let app = gtk::Application::builder().application_id(APP_ID).build();
    app.connect_activate(|app| {
        if let Some(window) = app.active_window() {
            window.present();
            return;
        }
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("CCTV View")
            .default_width(1280)
            .default_height(800)
            .build();
        let loading = gtk::Label::new(Some("Carregando biblioteca…"));
        window.set_titlebar(Some(&gtk::HeaderBar::new()));
        window.set_child(Some(&loading));
        window.present();
        let weak = window.downgrade();
        jobs::run(
            || -> Result<_> {
                let path = if let Some(p) = std::env::var_os("CCTV_VIEW_DATA_DIR") {
                    let p = PathBuf::from(p);
                    std::fs::create_dir_all(&p)?;
                    p.join("cctv.db")
                } else {
                    Store::default_path()?
                };
                let store = Store::open(&path)?;
                Ok((path, store.cameras()?, store.views()?, store.preferences()?))
            },
            move |result| {
                let Some(window) = weak.upgrade() else {
                    return;
                };
                match result {
                    Ok((path, cameras, views, prefs)) => match tokio::runtime::Runtime::new() {
                        Ok(runtime) => {
                            let _ = build(window, path, cameras, views, prefs, Arc::new(runtime));
                        }
                        Err(e) => {
                            message(&window, &format!("Não foi possível iniciar tarefas: {e}"))
                        }
                    },
                    Err(e) => message(
                        &window,
                        &format!("Não foi possível abrir a biblioteca: {e}"),
                    ),
                }
            },
        );
    });
    let quit = gio::SimpleAction::new("quit", None);
    let weak = app.downgrade();
    quit.connect_activate(move |_, _| {
        if let Some(a) = weak.upgrade() {
            for window in a.windows() {
                window.close();
            }
        }
    });
    app.add_action(&quit);
    app.set_accels_for_action("app.quit", &["<Primary>q"]);
    app.run();
    jobs::flush_preferences();
}
fn build(
    window: gtk::ApplicationWindow,
    path: PathBuf,
    cameras: Vec<Camera>,
    views: Vec<View>,
    prefs: Preferences,
    runtime: Arc<tokio::runtime::Runtime>,
) -> Shared {
    let css = gtk::CssProvider::new();
    css.load_from_data(include_str!("../../../resources/styles/player.css"));
    gtk::style_context_add_provider_for_display(
        &gtk::prelude::WidgetExt::display(&window),
        &css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    window.set_default_size(prefs.width.clamp(640, 7680), prefs.height.clamp(400, 4320));
    if prefs.maximized {
        window.maximize();
    }
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let header = window
        .titlebar()
        .and_then(|w| w.downcast::<gtk::HeaderBar>().ok())
        .unwrap_or_default();
    let heading = gtk::Label::new(Some("Monitoramento"));
    heading.add_css_class("title");
    header.set_title_widget(Some(&heading));
    let sidebar_toggle = gtk::ToggleButton::builder()
        .icon_name("sidebar-show-symbolic")
        .tooltip_text("Mostrar Views")
        .active(prefs.sidebar)
        .build();
    header.pack_start(&sidebar_toggle);
    let library = button("Câmeras", "camera-video-symbolic");
    header.pack_start(&library);
    let edit = button("Editar View", "document-edit-symbolic");
    header.pack_end(&edit);
    let fullscreen = button("Tela cheia · F11", "view-fullscreen-symbolic");
    header.pack_end(&fullscreen);
    let settings = button("Preferências", "emblem-system-symbolic");
    header.pack_end(&settings);
    if window.titlebar().is_none() {
        window.set_titlebar(Some(&header));
    }
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 12);
    sidebar.set_width_request(230);
    margins(&sidebar, 16);
    sidebar.set_visible(prefs.sidebar);
    let title = gtk::Label::new(Some("VIEWS"));
    title.set_xalign(0.);
    title.add_css_class("heading");
    sidebar.append(&title);
    let view_list = gtk::ListBox::new();
    view_list.add_css_class("navigation-sidebar");
    view_list.set_selection_mode(gtk::SelectionMode::None);
    let scroll = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .child(&view_list)
        .build();
    sidebar.append(&scroll);
    let new_view = button("Nova View", "list-add-symbolic");
    sidebar.append(&new_view);
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    let duplicate = button("Duplicar", "edit-copy-symbolic");
    let delete = button("Excluir View", "user-trash-symbolic");
    let favorite = button("Favorita", "starred-symbolic");
    let initial = button("View inicial", "go-home-symbolic");
    for b in [&duplicate, &favorite, &initial, &delete] {
        actions.append(b);
    }
    sidebar.append(&actions);
    let reorder = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    let up = button("Subir View", "go-up-symbolic");
    let down = button("Descer View", "go-down-symbolic");
    reorder.append(&up);
    reorder.append(&down);
    sidebar.append(&reorder);
    content.append(&sidebar);
    content.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    let grid = gtk::Grid::builder()
        .column_homogeneous(true)
        .row_homogeneous(true)
        .column_spacing(8)
        .row_spacing(8)
        .hexpand(true)
        .vexpand(true)
        .build();
    margins(&grid, 12);
    content.append(&grid);
    root.append(&content);
    content.set_vexpand(true);
    let status = gtk::Label::new(Some("Pronto · duplo clique para foco · Escape para voltar"));
    status.set_xalign(0.);
    margins(&status, 8);
    status.add_css_class("dim-label");
    root.append(&status);
    window.set_child(Some(&root));
    let selected = prefs
        .initial_view
        .clone()
        .or(prefs.last_view.clone())
        .filter(|id| views.iter().any(|v| &v.id == id))
        .or_else(|| views.first().map(|v| v.id.clone()));
    let state = Rc::new(State {
        window,
        cameras: RefCell::new(cameras),
        views: RefCell::new(views),
        preferences: RefCell::new(prefs),
        current: RefCell::new(selected),
        focus: RefCell::new(None),
        sidebar,
        view_list,
        grid,
        heading,
        status,
        path,
        manager: Arc::new(Mutex::new(SessionManager::default())),
        runtime,
    });
    if let Ok(m) = state.manager.lock() {
        m.audio.set_volume(state.preferences.borrow().volume);
    }
    let s = state.clone();
    sidebar_toggle.connect_toggled(move |b| {
        s.sidebar.set_visible(b.is_active());
        s.preferences.borrow_mut().sidebar = b.is_active();
        save_preferences(&s);
    });
    let s = state.clone();
    library.connect_clicked(move |_| camera::library(&s));
    let s = state.clone();
    new_view.connect_clicked(move |_| editor::open(&s, View::default()));
    let s = state.clone();
    edit.connect_clicked(move |_| {
        if let Some(v) = active_view(&s) {
            editor::open(&s, v);
        } else {
            editor::open(&s, View::default());
        }
    });
    let s = state.clone();
    duplicate.connect_clicked(move |_| {
        if let Some(v) = active_view(&s) {
            let mut copy = v.duplicate();
            copy.order = s.views.borrow().len() as i32;
            persist_view(&s, copy);
        }
    });
    let s = state.clone();
    favorite.connect_clicked(move |_| {
        if let Some(mut v) = active_view(&s) {
            v.favorite = !v.favorite;
            persist_view(&s, v);
        }
    });
    let s = state.clone();
    initial.connect_clicked(move |_| {
        s.preferences.borrow_mut().initial_view = s.current.borrow().clone();
        save_preferences(&s);
        s.status.set_text("View definida para a inicialização.");
    });
    let s = state.clone();
    delete.connect_clicked(move |_| {
        if let Some(v) = active_view(&s) {
            let s2 = s.clone();
            confirm(&s.window, "Excluir esta View?", move || {
                let path = s2.path.clone();
                let id = v.id.clone();
                let s3 = s2.clone();
                jobs::run(
                    move || Store::open(path)?.delete_view(&id),
                    move |r: Result<()>| match r {
                        Ok(()) => {
                            s3.views.borrow_mut().retain(|x| x.id != v.id);
                            *s3.current.borrow_mut() =
                                s3.views.borrow().first().map(|v| v.id.clone());
                            refresh(&s3);
                        }
                        Err(e) => message(&s3.window, &e.to_string()),
                    },
                );
            });
        }
    });
    for (b, delta) in [(up, -1), (down, 1)] {
        let s = state.clone();
        b.connect_clicked(move |_| {
            let list = s.views.borrow().clone();
            let Some(index) = list
                .iter()
                .position(|v| Some(&v.id) == s.current.borrow().as_ref())
            else {
                return;
            };
            let next = index as i32 + delta;
            if next < 0 || next >= list.len() as i32 {
                return;
            }
            let mut reordered = list;
            reordered.swap(index, next as usize);
            for (i, v) in reordered.iter_mut().enumerate() {
                v.order = i as i32;
            }
            let path = s.path.clone();
            let s2 = s.clone();
            jobs::run(
                move || -> Result<_> {
                    let store = Store::open(path)?;
                    for v in &reordered {
                        store.save_view(v)?;
                    }
                    Ok(reordered)
                },
                move |r| match r {
                    Ok(list) => {
                        *s2.views.borrow_mut() = list;
                        refresh_sidebar(&s2);
                    }
                    Err(e) => message(&s2.window, &e.to_string()),
                },
            );
        });
    }
    let s = state.clone();
    fullscreen.connect_clicked(move |_| toggle_fullscreen(&s.window));
    let s = state.clone();
    settings.connect_clicked(move |_| settings::open(&s));
    let keys = gtk::EventControllerKey::new();
    let weak_state = Rc::downgrade(&state);
    keys.connect_key_pressed(move |_, key, _, _| {
        let Some(s) = weak_state.upgrade() else {
            return glib::Propagation::Proceed;
        };
        match key {
            gdk::Key::Escape => {
                let had_focus = s.focus.borrow_mut().take().is_some();
                if had_focus {
                    render(&s);
                }
                s.window.unfullscreen();
            }
            gdk::Key::F11 => toggle_fullscreen(&s.window),
            _ => return glib::Propagation::Proceed,
        }
        glib::Propagation::Stop
    });
    state.window.add_controller(keys);
    let s = state.clone();
    let header_controls: Vec<gtk::Widget> = vec![
        sidebar_toggle.upcast(),
        library.upcast(),
        edit.upcast(),
        fullscreen.upcast(),
        settings.upcast(),
    ];
    let close_id = Rc::new(RefCell::new(None));
    let close_id_in_callback = close_id.clone();
    let handler = state.window.connect_close_request(move |w| {
        let mut p = s.preferences.borrow_mut();
        if !w.is_maximized() && !w.is_fullscreen() {
            p.width = w.width();
            p.height = w.height();
        }
        p.maximized = w.is_maximized();
        drop(p);
        save_preferences(&s);
        if let Ok(mut m) = s.manager.lock() {
            m.shutdown();
        }
        // State holds grid/sidebar references; explicitly break their signal ownership cycles.
        while let Some(child) = s.grid.first_child() {
            s.grid.remove(&child);
        }
        while let Some(child) = s.view_list.first_child() {
            s.view_list.remove(&child);
        }
        while let Some(child) = s.sidebar.first_child() {
            s.sidebar.remove(&child);
        }
        // Tear down child callbacks before releasing the last state owner.
        w.set_child(None::<&gtk::Widget>);
        if let Some(titlebar) = w.titlebar() {
            if let Ok(header) = titlebar.downcast::<gtk::HeaderBar>() {
                // Unparent packed controls while keeping the realized titlebar itself intact.
                if let Some(child) = header.title_widget() {
                    header.set_title_widget(None::<&gtk::Widget>);
                    drop(child);
                }
                for child in &header_controls {
                    header.remove(child);
                }
            }
        }
        if let Some(id) = close_id_in_callback.borrow_mut().take() {
            w.disconnect(id);
        }
        glib::Propagation::Proceed
    });
    *close_id.borrow_mut() = Some(handler);
    refresh(&state);
    // Optional non-interactive lifecycle smoke run, uses a temporary data directory in tests.
    if let Ok(ms) = std::env::var("CCTV_VIEW_SMOKE_MS") {
        if let Ok(ms) = ms.parse::<u64>() {
            let w = state.window.downgrade();
            glib::timeout_add_local_once(Duration::from_millis(ms), move || {
                if let Some(w) = w.upgrade() {
                    w.close();
                }
            });
        }
    }
    state
}
pub(crate) fn button(label: &str, icon: &str) -> gtk::Button {
    gtk::Button::builder()
        .icon_name(icon)
        .tooltip_text(label)
        .build()
}
pub(crate) fn margins(widget: &impl IsA<gtk::Widget>, n: i32) {
    widget.set_margin_top(n);
    widget.set_margin_bottom(n);
    widget.set_margin_start(n);
    widget.set_margin_end(n);
}
pub(crate) fn message(parent: &impl IsA<gtk::Window>, text: &str) {
    let d = gtk::MessageDialog::builder()
        .transient_for(parent)
        .modal(true)
        .message_type(gtk::MessageType::Info)
        .buttons(gtk::ButtonsType::Close)
        .text(text)
        .build();
    d.connect_response(|d, _| d.close());
    d.present();
}
pub(crate) fn confirm(parent: &impl IsA<gtk::Window>, text: &str, action: impl FnOnce() + 'static) {
    let d = gtk::MessageDialog::builder()
        .transient_for(parent)
        .modal(true)
        .message_type(gtk::MessageType::Question)
        .buttons(gtk::ButtonsType::YesNo)
        .text(text)
        .build();
    let action = RefCell::new(Some(action));
    d.connect_response(move |d, r| {
        if r == gtk::ResponseType::Yes {
            if let Some(f) = action.borrow_mut().take() {
                f();
            }
        }
        d.close();
    });
    d.present();
}
fn toggle_fullscreen(w: &gtk::ApplicationWindow) {
    if w.is_fullscreen() {
        w.unfullscreen();
    } else {
        w.fullscreen();
    }
}
pub(crate) fn active_view(s: &Shared) -> Option<View> {
    s.views
        .borrow()
        .iter()
        .find(|v| Some(&v.id) == s.current.borrow().as_ref())
        .cloned()
}
pub(crate) fn save_preferences(s: &Shared) {
    let path = s.path.clone();
    let p = s.preferences.borrow().clone();
    let weak = Rc::downgrade(s);
    jobs::preferences(
        move || Store::open(path)?.save_preferences(&p),
        move |r: Result<()>| {
            if let (Err(e), Some(s)) = (r, weak.upgrade()) {
                s.status
                    .set_text(&format!("Falha ao salvar preferências: {e}"));
            }
        },
    );
}
pub(crate) fn persist_view(s: &Shared, view: View) {
    let path = s.path.clone();
    let s = s.clone();
    jobs::run(
        move || -> Result<_> {
            Store::open(path)?.save_view(&view)?;
            Ok(view)
        },
        move |r| match r {
            Ok(v) => {
                *s.current.borrow_mut() = Some(v.id.clone());
                let mut views = s.views.borrow_mut();
                if let Some(old) = views.iter_mut().find(|x| x.id == v.id) {
                    *old = v;
                } else {
                    views.push(v);
                }
                views.sort_by_key(|v| v.order);
                drop(views);
                refresh(&s);
            }
            Err(e) => message(&s.window, &e.to_string()),
        },
    );
}
pub(crate) fn refresh(s: &Shared) {
    refresh_sidebar(s);
    render(s);
}
fn refresh_sidebar(s: &Shared) {
    while let Some(c) = s.view_list.first_child() {
        s.view_list.remove(&c);
    }
    for v in s.views.borrow().iter() {
        let label = format!("{}{}", if v.favorite { "★  " } else { "" }, v.name);
        let b = gtk::Button::with_label(&label);
        b.set_halign(gtk::Align::Fill);
        if Some(&v.id) == s.current.borrow().as_ref() {
            b.add_css_class("suggested-action");
        }
        let id = v.id.clone();
        let s2 = s.clone();
        b.connect_clicked(move |_| {
            *s2.current.borrow_mut() = Some(id.clone());
            s2.focus.borrow_mut().take();
            refresh(&s2);
        });
        s.view_list.append(&b);
    }
}
#[cfg(test)]
mod tests;
