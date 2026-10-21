mod auth;
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
    focus_bar: glib::WeakRef<gtk::Button>,
    presentation: std::cell::Cell<bool>,
    locked: std::cell::Cell<bool>,
    header: glib::WeakRef<gtk::HeaderBar>,
    separator: glib::WeakRef<gtk::Separator>,
    sidebar_toggle: glib::WeakRef<gtk::ToggleButton>,
    path: PathBuf,
    manager: Arc<Mutex<SessionManager>>,
    runtime: Arc<tokio::runtime::Runtime>,
}
#[derive(Default)]
pub struct LaunchOptions {
    pub view: Option<String>,
    pub fullscreen: bool,
    pub data_dir: Option<PathBuf>,
    pub unlock_password: Option<String>,
}
pub fn run() {
    run_with_options(LaunchOptions::default());
}
pub fn run_with_options(options: LaunchOptions) {
    let options = Rc::new(RefCell::new(options));
    let separate = options.borrow().view.is_some()
        || options.borrow().fullscreen
        || options.borrow().data_dir.is_some()
        || options.borrow().unlock_password.is_some();
    let app = gtk::Application::builder()
        .application_id(APP_ID)
        .flags(if separate {
            gio::ApplicationFlags::NON_UNIQUE
        } else {
            gio::ApplicationFlags::FLAGS_NONE
        })
        .build();
    app.connect_activate(move |app| {
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
        let target = options.borrow().view.clone();
        let fullscreen = options.borrow().fullscreen;
        let data_dir = options.borrow().data_dir.clone();
        let password = options.borrow_mut().unlock_password.take();
        jobs::run(
            move || -> Result<_> {
                let path = if let Some(p) =
                    data_dir.or_else(|| std::env::var_os("CCTV_VIEW_DATA_DIR").map(PathBuf::from))
                {
                    std::fs::create_dir_all(&p)?;
                    p.join("cctv.db")
                } else {
                    Store::default_path()?
                };
                let store = Store::open(&path)?;
                let locked = store.has_app_password()?
                    && !password
                        .as_deref()
                        .map(|p| store.verify_app_password(p))
                        .transpose()?
                        .unwrap_or(false);
                Ok((
                    path,
                    store.cameras()?,
                    store.views()?,
                    store.preferences()?,
                    locked,
                ))
            },
            move |result| {
                let Some(window) = weak.upgrade() else {
                    return;
                };
                match result {
                    Ok((path, cameras, views, prefs, locked)) => {
                        match tokio::runtime::Runtime::new() {
                            Ok(runtime) => {
                                let weak = window.downgrade();
                                let gate_path = path.clone();
                                let start = move || {
                                    if let Some(window) = weak.upgrade() {
                                        let state = build(
                                            window,
                                            path,
                                            cameras,
                                            views,
                                            prefs,
                                            Arc::new(runtime),
                                        );
                                        if let Some(target) = target {
                                            let found = state
                                                .views
                                                .borrow()
                                                .iter()
                                                .find(|v| v.id == target || v.name == target)
                                                .map(|v| v.id.clone());
                                            if let Some(id) = found {
                                                *state.current.borrow_mut() = Some(id);
                                                refresh(&state);
                                            } else {
                                                message(
                                                    &state.window,
                                                    "View solicitada não encontrada.",
                                                );
                                            }
                                        }
                                        if fullscreen {
                                            set_fullscreen(&state, true);
                                        }
                                    }
                                };
                                if locked {
                                    auth::gate(&window, gate_path, start);
                                } else {
                                    start();
                                }
                            }
                            Err(e) => {
                                message(&window, &format!("Não foi possível iniciar tarefas: {e}"))
                            }
                        }
                    }
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
    app.run_with_args(&["cctv-view"]);
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
    view_list.add_css_class("navigation-sidebar");
    view_list.set_selection_mode(gtk::SelectionMode::Single);
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
    let focus_bar = button("Voltar à grade", "window-close-symbolic");
    focus_bar.set_halign(gtk::Align::End);
    focus_bar.set_valign(gtk::Align::Start);
    margins(&focus_bar, 12);
    focus_bar.set_size_request(36, 36);
    focus_bar.add_css_class("cctv-floating-close");
    focus_bar.set_visible(false);
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
    let stage = gtk::Overlay::new();
    stage.set_child(Some(&content));
    stage.add_overlay(&focus_bar);
    stage.set_vexpand(true);
    root.append(&stage);
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
        presentation: std::cell::Cell::new(false),
        locked: std::cell::Cell::new(false),
        header: header.downgrade(),
        separator: separator.downgrade(),
        sidebar_toggle: sidebar_toggle.downgrade(),
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
        s.sidebar
            .set_visible(b.is_active() && !s.presentation.get());
        sep.set_visible(b.is_active() && !s.presentation.get());
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
        if s.presentation.get() {
            return glib::ControlFlow::Continue;
        }
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
    let weak = Rc::downgrade(&state);
    state.window.connect_fullscreened_notify(move |window| {
        if let Some(s) = weak.upgrade() {
            if s.presentation.replace(window.is_fullscreen()) != window.is_fullscreen() {
                render(&s);
            }
        }
    });
    let weak_state = Rc::downgrade(&state);
    state.view_list.connect_row_activated(move |_, row| {
        let Some(s) = weak_state.upgrade() else {
            return;
        };
        let view = s.views.borrow().get(row.index() as usize).cloned();
        if let Some(view) = view {
            *s.current.borrow_mut() = Some(view.id);
            s.focus.borrow_mut().take();
            s.view_list.select_row(Some(row));
            render(&s);
        }
    });
    let s = state.clone();
    focus_bar.connect_clicked(move |_| {
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
    fullscreen.connect_clicked(move |_| set_fullscreen(&s, !s.presentation.get()));
    let s = state.clone();
    settings.connect_clicked(move |_| settings::open(&s));
    let keys = gtk::EventControllerKey::new();
    let weak_state = Rc::downgrade(&state);
    keys.connect_key_pressed(move |_, key, _, _| {
        let Some(s) = weak_state.upgrade() else {
            return glib::Propagation::Proceed;
        };
        if s.locked.get() {
            return glib::Propagation::Proceed;
        }
        match key {
            gdk::Key::Escape => {
                let had_focus = s.focus.borrow_mut().take().is_some();
                if s.presentation.get() {
                    set_fullscreen(&s, false);
                } else if had_focus {
                    render(&s);
                }
            }
            gdk::Key::F11 => set_fullscreen(&s, !s.presentation.get()),
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
fn set_fullscreen(s: &Shared, active: bool) {
    s.presentation.set(active);
    if active {
        s.window.fullscreen();
    } else {
        s.window.unfullscreen();
    }
    render(s);
}
fn update_chrome(s: &Shared) {
    let active = s.presentation.get();
    if let Some(header) = s.header.upgrade() {
        header.set_visible(!active);
    }
    s.window.set_decorated(!active);
    let show_sidebar = !active && s.sidebar_toggle.upgrade().is_some_and(|b| b.is_active());
    s.sidebar.set_visible(show_sidebar);
    if let Some(sep) = s.separator.upgrade() {
        sep.set_visible(show_sidebar);
    }
    if let Some(back) = s.focus_bar.upgrade() {
        back.set_visible(!active && s.focus.borrow().is_some());
    }
    s.status.set_visible(!active && !s.status.text().is_empty());
    margins(&s.grid, if active { 0 } else { 12 });
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
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let name = gtk::Label::new(Some(&v.name));
        name.set_xalign(0.);
        name.set_hexpand(true);
        name.set_ellipsize(gtk::pango::EllipsizeMode::End);
        name.set_width_chars(1);
        row.append(&name);
        if v.favorite {
            let favorite = gtk::Image::from_icon_name("starred-symbolic");
            favorite.set_pixel_size(16);
            favorite.set_tooltip_text(Some("Favorita"));
            row.append(&favorite);
        }
        let initial = s.preferences.borrow().initial_view.as_ref() == Some(&v.id);
        if initial {
            let badge = gtk::Label::new(Some("Inicial"));
            badge.add_css_class("dim-label");
            row.append(&badge);
        }
        let actions = views::menu(s, v, index);
        actions.set_has_frame(false);
        actions.set_valign(gtk::Align::Center);
        row.append(&actions);
        let item = gtk::ListBoxRow::new();
        item.set_child(Some(&row));
        item.set_tooltip_text(Some(&v.name));
        item.update_property(&[gtk::accessible::Property::Label(&format!(
            "Abrir View {}{}{}",
            v.name,
            if v.favorite { ", favorita" } else { "" },
            if initial { ", inicial" } else { "" }
        ))]);
        s.view_list.append(&item);
        if s.current.borrow().as_ref() == Some(&v.id) {
            s.view_list.select_row(Some(&item));
        }
    }
}
#[cfg(test)]
mod tests;
