mod camera;
mod diagnostics;
mod editor;
mod forms;
mod jobs;
mod monitor;
mod player;
mod settings;
mod views;
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
    edit_view: glib::WeakRef<gtk::Button>,
    focus_bar: glib::WeakRef<gtk::Box>,
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
    heading.set_ellipsize(gtk::pango::EllipsizeMode::End);
    heading.set_max_width_chars(20);
    header.set_title_widget(Some(&heading));
    let sidebar_toggle = gtk::ToggleButton::builder()
        .icon_name("sidebar-show-symbolic")
        .tooltip_text("Mostrar Views")
        .active(prefs.sidebar)
        .build();
    header.pack_start(&sidebar_toggle);
    let library = gtk::Button::with_label("Câmeras");
    header.pack_start(&library);
    let edit = gtk::Button::with_label("Editar View");
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
    view_list.add_css_class("background");
    view_list.set_selection_mode(gtk::SelectionMode::None);
    let scroll = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&view_list)
        .build();
    sidebar.append(&scroll);
    let new_view = gtk::Button::with_label("Nova View");
    new_view.add_css_class("suggested-action");
    sidebar.append(&new_view);
    content.append(&sidebar);
    let separator = gtk::Separator::new(gtk::Orientation::Vertical);
    content.append(&separator);
    let focus_bar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    margins(&focus_bar, 8);
    let back = gtk::Button::with_label("Voltar à grade");
    focus_bar.append(&back);
    let focus_hint = gtk::Label::new(Some("Câmera em foco · Escape também retorna à grade"));
    focus_hint.set_wrap(true);
    focus_bar.append(&focus_hint);
    focus_bar.set_visible(false);
    root.append(&focus_bar);
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
    let status = gtk::Label::new(None);
    status.set_xalign(0.);
    status.set_wrap(true);
    margins(&status, 8);
    status.set_visible(false);
    status.connect_label_notify(|label| label.set_visible(!label.text().is_empty()));
    root.append(&status);
    window.set_child(Some(&root));
    let selected = prefs
        .initial_view
        .clone()
        .filter(|id| views.iter().any(|v| &v.id == id))
        .or_else(|| {
            prefs
                .last_view
                .clone()
                .filter(|id| views.iter().any(|v| &v.id == id))
        })
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
        edit_view: edit.downgrade(),
        focus_bar: focus_bar.downgrade(),
        path,
        manager: Arc::new(Mutex::new(SessionManager::default())),
        runtime,
    });
    if let Ok(m) = state.manager.lock() {
        m.audio.set_volume(state.preferences.borrow().volume);
    }
    let automatic = Rc::new(std::cell::Cell::new(false));
    let changing = automatic.clone();
    let s = state.clone();
    let sep = separator.clone();
    sidebar_toggle.connect_toggled(move |b| {
        s.sidebar.set_visible(b.is_active());
        sep.set_visible(b.is_active());
        if changing.get() {
            return;
        }
        s.preferences.borrow_mut().sidebar = b.is_active();
        save_preferences(&s);
    });
    let weak_state = Rc::downgrade(&state);
    let weak_toggle = sidebar_toggle.downgrade();
    let compact = std::cell::Cell::new(None);
    state.window.add_tick_callback(move |window, _| {
        let Some(s) = weak_state.upgrade() else {
            return glib::ControlFlow::Break;
        };
        let now = window.width() < 900;
        if compact.get() != Some(now) {
            compact.set(Some(now));
            if let Some(toggle) = weak_toggle.upgrade() {
                automatic.set(true);
                toggle.set_active(!now && s.preferences.borrow().sidebar);
                s.sidebar.set_visible(toggle.is_active());
                separator.set_visible(toggle.is_active());
                automatic.set(false);
            }
        }
        glib::ControlFlow::Continue
    });
    let s = state.clone();
    back.connect_clicked(move |_| {
        s.focus.borrow_mut().take();
        render(&s);
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
    let button = gtk::Button::builder()
        .icon_name(icon)
        .tooltip_text(label)
        .build();
    button.update_property(&[gtk::accessible::Property::Label(label)]);
    button
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
pub(crate) fn persist_view(s: &Shared, view: View, activate: bool) {
    let path = s.path.clone();
    let s = s.clone();
    jobs::run(
        move || -> Result<_> {
            Store::open(path)?.save_view(&view)?;
            Ok(view)
        },
        move |r| match r {
            Ok(v) => {
                if activate {
                    *s.current.borrow_mut() = Some(v.id.clone());
                    s.focus.borrow_mut().take();
                }
                let mut views = s.views.borrow_mut();
                if let Some(old) = views.iter_mut().find(|x| x.id == v.id) {
                    *old = v;
                } else {
                    views.push(v);
                }
                views.sort_by_key(|v| v.order);
                drop(views);
                if activate {
                    refresh(&s);
                } else {
                    refresh_sidebar(&s);
                }
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
    if let Some(edit) = s.edit_view.upgrade() {
        edit.set_sensitive(active_view(s).is_some());
    }
    let views = s.views.borrow().clone();
    for (index, v) in views.iter().enumerate() {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let select = gtk::Button::new();
        select.set_hexpand(true);
        select.set_height_request(60);
        select.set_valign(gtk::Align::Center);
        select.set_tooltip_text(Some(&v.name));
        let content = gtk::Box::new(gtk::Orientation::Vertical, 4);
        content.set_valign(gtk::Align::Center);
        content.set_hexpand(true);
        let name = gtk::Label::new(Some(&v.name));
        name.set_xalign(0.);
        name.set_ellipsize(gtk::pango::EllipsizeMode::End);
        name.set_max_width_chars(20);
        content.append(&name);
        let mut badges = vec![];
        if v.favorite {
            badges.push("★ Favorita");
        }
        if s.preferences.borrow().initial_view.as_ref() == Some(&v.id) {
            badges.push("Inicial");
        }
        // Reserve the same subtitle line so names and menus align with or without badges.
        let subtitle = if badges.is_empty() {
            " ".to_string()
        } else {
            badges.join(" · ")
        };
        let label = gtk::Label::new(Some(&subtitle));
        label.set_xalign(0.);
        label.add_css_class("dim-label");
        content.append(&label);
        select.set_child(Some(&content));
        select.update_property(&[gtk::accessible::Property::Label(&format!(
            "Abrir View {}",
            v.name
        ))]);
        if s.current.borrow().as_ref() == Some(&v.id) {
            select.add_css_class("suggested-action");
        }
        let s2 = s.clone();
        let id = v.id.clone();
        select.connect_clicked(move |_| {
            *s2.current.borrow_mut() = Some(id.clone());
            s2.focus.borrow_mut().take();
            refresh(&s2);
        });
        row.append(&select);
        let actions = views::menu(s, v, index);
        actions.set_valign(gtk::Align::Center);
        actions.set_size_request(36, 36);
        row.append(&actions);
        let item = gtk::ListBoxRow::new();
        item.set_activatable(false);
        item.set_selectable(false);
        item.set_child(Some(&row));
        item.set_margin_bottom(4);
        s.view_list.append(&item);
    }
}
#[cfg(test)]
mod tests;
