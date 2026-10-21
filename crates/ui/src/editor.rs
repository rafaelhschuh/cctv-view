use super::*;
use cctv_core::{Aspect, Panel};
#[derive(Clone)]
struct Editor {
    state: Shared,
    draft: Rc<RefCell<View>>,
    grid: gtk::Grid,
    feedback: gtk::Label,
    preview: Rc<std::cell::Cell<bool>>,
}
pub fn open(s: &Shared, view: View) {
    let d = gtk::Dialog::builder()
        .transient_for(&s.window)
        .destroy_with_parent(true)
        .modal(true)
        .title("Editor de Views")
        .default_width(1100)
        .default_height(740)
        .build();
    d.add_button("Cancelar", gtk::ResponseType::Cancel);
    d.add_button("Salvar View", gtk::ResponseType::Accept)
        .add_css_class("suggested-action");
    let root = gtk::Box::new(gtk::Orientation::Vertical, 12);
    margins(&root, 16);
    d.content_area().append(&root);
    let top = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let name = gtk::Entry::builder()
        .text(&view.name)
        .placeholder_text("Nome da View")
        .hexpand(true)
        .build();
    top.append(&name);
    let preset = gtk::DropDown::from_strings(&[
        "Personalizado",
        "1 câmera",
        "2 câmeras",
        "2 × 2",
        "3 × 3",
        "4 × 4",
        "Principal + secundárias",
    ]);
    top.append(&preset);
    let columns = gtk::SpinButton::with_range(1., 16., 1.);
    columns.set_value(view.columns as f64);
    columns.set_tooltip_text(Some("Colunas"));
    top.append(&gtk::Label::new(Some("Colunas")));
    top.append(&columns);
    let rows = gtk::SpinButton::with_range(1., 16., 1.);
    rows.set_value(view.rows as f64);
    rows.set_tooltip_text(Some("Linhas"));
    top.append(&gtk::Label::new(Some("Linhas")));
    top.append(&rows);
    let preview = gtk::ToggleButton::with_label("Visualizar");
    top.append(&preview);
    root.append(&top);
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 16);
    content.set_vexpand(true);
    root.append(&content);
    let library = gtk::Box::new(gtk::Orientation::Vertical, 8);
    library.set_width_request(180);
    library.append(&gtk::Label::new(Some("ARRASTE PARA A GRADE")));
    for c in s.cameras.borrow().iter() {
        let b = gtk::Button::with_label(&c.name);
        b.set_tooltip_text(Some("Arraste para adicionar uma câmera"));
        let drag = gtk::DragSource::new();
        drag.set_actions(gdk::DragAction::COPY);
        let id = format!("camera:{}", c.id);
        drag.connect_prepare(move |_, _, _| Some(gdk::ContentProvider::for_value(&id.to_value())));
        b.add_controller(drag);
        library.append(&b);
    }
    let scroll = gtk::ScrolledWindow::builder()
        .child(&library)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();
    content.append(&scroll);
    let grid = gtk::Grid::builder()
        .column_homogeneous(true)
        .row_homogeneous(true)
        .column_spacing(6)
        .row_spacing(6)
        .hexpand(true)
        .vexpand(true)
        .build();
    content.append(&grid);
    let feedback=gtk::Label::new(Some("Arraste câmeras ou painéis. Clique no painel para trocar câmera, proporção, posição e tamanho. Arraste ↘ para redimensionar."));
    feedback.set_wrap(true);
    feedback.set_xalign(0.);
    root.append(&feedback);
    let editor = Editor {
        state: s.clone(),
        draft: Rc::new(RefCell::new(view)),
        grid,
        feedback,
        preview: Rc::new(std::cell::Cell::new(false)),
    };
    draw(&editor);
    let e = editor.clone();
    preset.connect_selected_notify(move |p| {
        let (c, r, main) = match p.selected() {
            1 => (1, 1, false),
            2 => (2, 1, false),
            3 => (2, 2, false),
            4 => (3, 3, false),
            5 => (4, 4, false),
            6 => (3, 3, true),
            _ => return,
        };
        let ids: Vec<_> = e
            .state
            .cameras
            .borrow()
            .iter()
            .map(|c| c.id.clone())
            .collect();
        e.draft.borrow_mut().preset(c, r, &ids, main);
        columns.set_value(c as f64);
        rows.set_value(r as f64);
        draw(&e);
    });
    // Grid dimensions are applied explicitly to avoid destroying panels on intermediate spin changes.
    let resize = gtk::Button::with_label("Aplicar grade");
    top.append(&resize);
    // Locate the numeric controls already parented in the toolbar; retain their typed handles.
    let numeric: Vec<gtk::SpinButton> = children(&top)
        .into_iter()
        .filter_map(|w| w.downcast().ok())
        .collect();
    let e = editor.clone();
    resize.connect_clicked(move |_| {
        if numeric.len() != 2 {
            return;
        }
        let mut candidate = e.draft.borrow().clone();
        candidate.columns = numeric[0].value_as_int() as u8;
        candidate.rows = numeric[1].value_as_int() as u8;
        match candidate.validate() {
            Ok(()) => {
                *e.draft.borrow_mut() = candidate;
                draw(&e);
            }
            Err(err) => e.feedback.set_text(&err.to_string()),
        }
    });
    let e = editor.clone();
    preview.connect_toggled(move |button| {
        e.preview.set(button.is_active());
        draw(&e);
    });
    let e = editor.clone();
    let s = s.clone();
    let cleanup = editor.clone();
    d.connect_close_request(move |_| {
        clear_grid(&cleanup.grid);
        glib::Propagation::Proceed
    });
    d.connect_response(move |d, response| {
        if response != gtk::ResponseType::Accept {
            clear_grid(&e.grid);
            d.close();
            return;
        }
        let mut view = e.draft.borrow().clone();
        view.name = name.text().to_string();
        if !s.views.borrow().iter().any(|v| v.id == view.id) {
            view.order = s.views.borrow().len() as i32;
        }
        if let Err(err) = view.validate() {
            e.feedback.set_text(&err.to_string());
            return;
        }
        let path = s.path.clone();
        let s2 = s.clone();
        let weak = d.downgrade();
        let feedback = e.feedback.clone();
        let cleanup_grid = e.grid.clone();
        jobs::run(
            move || -> Result<_> {
                let store = Store::open(path)?;
                store.save_view(&view)?;
                Ok(view)
            },
            move |r| match r {
                Ok(view) => {
                    *s2.current.borrow_mut() = Some(view.id.clone());
                    let mut views = s2.views.borrow_mut();
                    if let Some(old) = views.iter_mut().find(|v| v.id == view.id) {
                        *old = view;
                    } else {
                        views.push(view);
                    }
                    drop(views);
                    refresh(&s2);
                    clear_grid(&cleanup_grid);
                    if let Some(d) = weak.upgrade() {
                        d.close();
                    }
                }
                Err(err) => feedback.set_text(&err.to_string()),
            },
        );
    });
    d.present();
}
fn children(w: &gtk::Box) -> Vec<gtk::Widget> {
    let mut result = vec![];
    let mut next = w.first_child();
    while let Some(c) = next {
        next = c.next_sibling();
        result.push(c);
    }
    result
}
fn draw(e: &Editor) {
    while let Some(c) = e.grid.first_child() {
        e.grid.remove(&c);
    }
    let view = e.draft.borrow().clone();
    let preview = e.preview.get();
    for y in 0..view.rows {
        for x in 0..view.columns {
            if view
                .panels
                .iter()
                .any(|p| x >= p.x && x < p.x + p.width && y >= p.y && y < p.y + p.height)
            {
                continue;
            }
            let b = gtk::Button::with_label("+");
            b.set_tooltip_text(Some("Solte uma câmera aqui"));
            b.set_hexpand(true);
            b.set_vexpand(true);
            b.set_size_request(50, 45);
            b.set_sensitive(!preview);
            drop_target(e, &b, x, y);
            e.grid.attach(&b, x as i32, y as i32, 1, 1);
        }
    }
    for p in view.panels {
        let camera = e
            .state
            .cameras
            .borrow()
            .iter()
            .find(|c| c.id == p.camera_id)
            .cloned();
        if preview {
            if let Some(c) = camera {
                let t = tile(&e.state, &c, p.aspect, false);
                e.grid
                    .attach(&t, p.x as i32, p.y as i32, p.width as i32, p.height as i32);
            }
            continue;
        }
        let container = gtk::Box::new(gtk::Orientation::Vertical, 4);
        container.add_css_class("card");
        container.set_hexpand(true);
        container.set_vexpand(true);
        let b = gtk::Button::with_label(&format!(
            "{}\n{} × {}",
            camera
                .map(|c| c.name)
                .unwrap_or_else(|| "Câmera removida".into()),
            p.width,
            p.height
        ));
        b.set_hexpand(true);
        b.set_vexpand(true);
        container.append(&b);
        let e2 = e.clone();
        let id = p.id.clone();
        b.connect_clicked(move |_| panel_settings(&e2, &id));
        let source = gtk::DragSource::new();
        source.set_actions(gdk::DragAction::MOVE);
        let id = format!("panel:{}", p.id);
        source
            .connect_prepare(move |_, _, _| Some(gdk::ContentProvider::for_value(&id.to_value())));
        b.add_controller(source);
        drop_target(e, &b, p.x, p.y);
        let handle = gtk::Label::new(Some("↘"));
        handle.set_halign(gtk::Align::End);
        handle.set_tooltip_text(Some("Arraste para redimensionar com snap à grade"));
        container.append(&handle);
        let drag = gtk::GestureDrag::new();
        let e2 = e.clone();
        let id = p.id.clone();
        drag.connect_drag_end(move |_, dx, dy| {
            let mut v = e2.draft.borrow().clone();
            let cw = e2.grid.width() as f64 / v.columns as f64;
            let ch = e2.grid.height() as f64 / v.rows as f64;
            if let Some(p) = v.panels.iter_mut().find(|p| p.id == id) {
                p.width = (p.width as f64 + dx / cw.max(1.)).round().clamp(1., 16.) as u8;
                p.height = (p.height as f64 + dy / ch.max(1.)).round().clamp(1., 16.) as u8;
            }
            apply(&e2, v);
        });
        handle.add_controller(drag);
        e.grid.attach(
            &container,
            p.x as i32,
            p.y as i32,
            p.width as i32,
            p.height as i32,
        );
    }
}
fn apply(e: &Editor, v: View) -> bool {
    match v.validate() {
        Ok(()) => {
            *e.draft.borrow_mut() = v;
            e.feedback
                .set_text("Layout atualizado. Salve a View para manter as alterações.");
            draw(e);
            true
        }
        Err(err) => {
            e.feedback.set_text(&err.to_string());
            false
        }
    }
}
fn drop_target(e: &Editor, widget: &impl IsA<gtk::Widget>, x: u8, y: u8) {
    let target = gtk::DropTarget::new(
        String::static_type(),
        gdk::DragAction::COPY | gdk::DragAction::MOVE,
    );
    let e = e.clone();
    target.connect_drop(move |_, value, _, _| {
        let Ok(text) = value.get::<String>() else {
            return false;
        };
        let mut v = e.draft.borrow().clone();
        if let Some(id) = text.strip_prefix("camera:") {
            if !e.state.cameras.borrow().iter().any(|c| c.id == id) {
                return false;
            }
            if let Some(p) = v.panels.iter_mut().find(|p| p.x == x && p.y == y) {
                p.camera_id = id.into();
            } else {
                v.panels.push(Panel {
                    id: cctv_core::id(),
                    camera_id: id.into(),
                    x,
                    y,
                    width: 1,
                    height: 1,
                    aspect: Aspect::Fit,
                });
            }
        } else if let Some(id) = text.strip_prefix("panel:") {
            let Some(index) = v.panels.iter().position(|p| p.id == id) else {
                return false;
            };
            let old = (v.panels[index].x, v.panels[index].y);
            if let Some(other) = v
                .panels
                .iter()
                .position(|p| p.x == x && p.y == y && p.id != id)
            {
                v.panels[other].x = old.0;
                v.panels[other].y = old.1;
            }
            v.panels[index].x = x;
            v.panels[index].y = y;
        } else {
            return false;
        }
        apply(&e, v)
    });
    widget.add_controller(target);
}
fn panel_settings(e: &Editor, id: &str) {
    let view = e.draft.borrow().clone();
    let Some(p) = view.panels.iter().find(|p| p.id == id) else {
        return;
    };
    let d = gtk::Dialog::builder()
        .transient_for(&e.state.window)
        .destroy_with_parent(true)
        .modal(true)
        .title("Configurar painel")
        .build();
    d.add_button("Remover", gtk::ResponseType::Reject);
    d.add_button("Cancelar", gtk::ResponseType::Cancel);
    d.add_button("Aplicar", gtk::ResponseType::Accept);
    let grid = gtk::Grid::builder()
        .row_spacing(12)
        .column_spacing(16)
        .build();
    margins(&grid, 20);
    d.content_area().append(&grid);
    let cameras = e.state.cameras.borrow().clone();
    let dropdown =
        gtk::DropDown::from_strings(&cameras.iter().map(|c| c.name.as_str()).collect::<Vec<_>>());
    dropdown.set_selected(
        cameras
            .iter()
            .position(|c| c.id == p.camera_id)
            .unwrap_or(0) as u32,
    );
    grid.attach(&gtk::Label::new(Some("Câmera")), 0, 0, 1, 1);
    grid.attach(&dropdown, 1, 0, 1, 1);
    let aspect =
        gtk::DropDown::from_strings(&["Manter proporção", "Preencher e recortar", "Esticar"]);
    aspect.set_selected(match p.aspect {
        Aspect::Fit => 0,
        Aspect::Fill => 1,
        Aspect::Stretch => 2,
    });
    grid.attach(&gtk::Label::new(Some("Imagem")), 0, 1, 1, 1);
    grid.attach(&aspect, 1, 1, 1, 1);
    let mut controls = vec![];
    for (i, (label, value, min)) in [
        ("Coluna", p.x, 0.),
        ("Linha", p.y, 0.),
        ("Largura", p.width, 1.),
        ("Altura", p.height, 1.),
    ]
    .into_iter()
    .enumerate()
    {
        let spin = gtk::SpinButton::with_range(min, 16., 1.);
        spin.set_value(value as f64);
        grid.attach(&gtk::Label::new(Some(label)), 0, i as i32 + 2, 1, 1);
        grid.attach(&spin, 1, i as i32 + 2, 1, 1);
        controls.push(spin);
    }
    let e = e.clone();
    let id = id.to_string();
    d.connect_response(move |d, r| {
        let mut v = e.draft.borrow().clone();
        match r {
            gtk::ResponseType::Reject => {
                v.panels.retain(|p| p.id != id);
                apply(&e, v);
                d.close();
            }
            gtk::ResponseType::Accept => {
                if let Some(p) = v.panels.iter_mut().find(|p| p.id == id) {
                    if let Some(c) = cameras.get(dropdown.selected() as usize) {
                        p.camera_id = c.id.clone();
                    }
                    p.x = controls[0].value_as_int() as u8;
                    p.y = controls[1].value_as_int() as u8;
                    p.width = controls[2].value_as_int() as u8;
                    p.height = controls[3].value_as_int() as u8;
                    p.aspect = match aspect.selected() {
                        1 => Aspect::Fill,
                        2 => Aspect::Stretch,
                        _ => Aspect::Fit,
                    };
                }
                if apply(&e, v) {
                    d.close();
                }
            }
            _ => d.close(),
        }
    });
    d.present();
}

fn clear_grid(grid: &gtk::Grid) {
    while let Some(child) = grid.first_child() {
        grid.remove(&child);
    }
}
