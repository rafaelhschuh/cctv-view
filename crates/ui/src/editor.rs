use super::*;
use cctv_core::{Aspect, Panel};
#[derive(Clone)]
struct Editor {
    state: Shared,
    draft: Rc<RefCell<View>>,
    grid: gtk::Grid,
    feedback: gtk::Label,
    preview: Rc<std::cell::Cell<bool>>,
    history: Rc<RefCell<Vec<View>>>,
    undo: glib::WeakRef<gtk::Button>,
    window: glib::WeakRef<gtk::Dialog>,
    columns: gtk::SpinButton,
    rows: gtk::SpinButton,
}
pub fn open(s: &Shared, mut view: View) {
    let conversion_error = view.scale_grid(12, 12).err();
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
    let name = gtk::Entry::builder()
        .text(&view.name)
        .placeholder_text("Nome da View")
        .hexpand(true)
        .build();
    forms::row(&root, "Nome da View", &name);
    let layout = gtk::FlowBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .min_children_per_line(2)
        .max_children_per_line(4)
        .column_spacing(12)
        .row_spacing(8)
        .build();
    let preset = gtk::DropDown::from_strings(&[
        "Personalizado",
        "1 câmera",
        "2 câmeras",
        "2 × 2",
        "3 × 3",
        "4 × 4",
        "Principal + secundárias",
    ]);
    let columns = gtk::SpinButton::with_range(1., 12., 1.);
    columns.set_value(4.);
    let rows = gtk::SpinButton::with_range(1., 12., 1.);
    rows.set_value(4.);
    for (label, control) in [
        ("Layout", preset.clone().upcast::<gtk::Widget>()),
        ("Largura do novo painel", columns.clone().upcast()),
        ("Altura do novo painel", rows.clone().upcast()),
    ] {
        let group = gtk::Box::new(gtk::Orientation::Vertical, 6);
        forms::row(&group, label, &control);
        layout.insert(&group, -1);
    }
    let resize = gtk::Button::with_label("Adicionar painel");
    resize.set_valign(gtk::Align::End);
    layout.insert(&resize, -1);
    root.append(&layout);
    let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let undo = gtk::Button::with_label("Desfazer layout");
    undo.set_sensitive(false);
    let fill = gtk::Button::with_label("Preencher espaços");
    fill.set_tooltip_text(Some(
        "Adicionar câmeras ainda não usadas, sem substituir os painéis existentes",
    ));
    let preview = gtk::ToggleButton::with_label("Visualizar");
    toolbar.append(&undo);
    toolbar.append(&fill);
    toolbar.append(&preview);
    root.append(&toolbar);
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    content.set_vexpand(true);
    root.append(&content);
    let library = gtk::Box::new(gtk::Orientation::Vertical, 8);
    library.set_width_request(160);
    forms::section(&library, "Câmeras");
    forms::help(
        &library,
        "Grade 12 × 12. Clique para adicionar ou arraste uma câmera. Desenhe um retângulo na grade para escolher tamanho e posição.",
    );
    let grid = gtk::Grid::builder()
        .column_homogeneous(true)
        .row_homogeneous(true)
        .column_spacing(3)
        .row_spacing(3)
        .hexpand(true)
        .vexpand(true)
        .build();
    let feedback = gtk::Label::new(Some(
        "Clique em + para escolher uma câmera. Clique num painel para editar suas propriedades.",
    ));
    feedback.set_wrap(true);
    feedback.set_xalign(0.);
    feedback.set_max_width_chars(80);
    let editor = Editor {
        state: s.clone(),
        draft: Rc::new(RefCell::new(view)),
        grid,
        feedback,
        preview: Rc::new(std::cell::Cell::new(false)),
        history: Rc::new(RefCell::new(vec![])),
        undo: undo.downgrade(),
        window: d.downgrade(),
        columns: columns.clone(),
        rows: rows.clone(),
    };
    for c in s.cameras.borrow().iter() {
        let b = gtk::Button::with_label(&c.name);
        b.set_tooltip_text(Some(&format!(
            "Adicionar {} ao primeiro espaço livre",
            c.name
        )));
        if let Some(label) = b.child().and_then(|w| w.downcast::<gtk::Label>().ok()) {
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            label.set_max_width_chars(20);
        }
        let drag = gtk::DragSource::new();
        drag.set_actions(gdk::DragAction::COPY);
        let data = format!("camera:{}", c.id);
        drag.connect_prepare(move |_, _, _| {
            Some(gdk::ContentProvider::for_value(&data.to_value()))
        });
        b.add_controller(drag);
        let e = editor.clone();
        let id = c.id.clone();
        b.connect_clicked(move |_| add_camera(&e, &id, None));
        library.append(&b);
    }
    let library_scroll = gtk::ScrolledWindow::builder()
        .child(&library)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();
    content.append(&library_scroll);
    let canvas_scroll = gtk::ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .min_content_width(240)
        .min_content_height(160)
        .build();
    let selection = Rc::new(std::cell::Cell::new(None::<(u8, u8, u8, u8)>));
    let canvas = gtk::Overlay::new();
    canvas.set_child(Some(&editor.grid));
    canvas_scroll.set_child(Some(&canvas));
    canvas.set_hexpand(true);
    canvas.set_vexpand(true);
    let outline = gtk::DrawingArea::new();
    outline.set_can_target(false);
    canvas.add_overlay(&outline);
    let selected = selection.clone();
    let dimensions = editor.draft.clone();
    outline.set_draw_func(move |_, cr, width, height| {
        if let Some((x, y, w, h)) = selected.get() {
            let v = dimensions.borrow();
            let cw = width as f64 / f64::from(v.columns);
            let ch = height as f64 / f64::from(v.rows);
            cr.set_source_rgba(0.2, 0.6, 1., 0.3);
            cr.rectangle(
                f64::from(x) * cw,
                f64::from(y) * ch,
                f64::from(w) * cw,
                f64::from(h) * ch,
            );
            let _ = cr.fill_preserve();
            cr.set_source_rgb(0.2, 0.6, 1.);
            cr.set_line_width(2.);
            let _ = cr.stroke();
        }
    });
    let drag = gtk::GestureDrag::new();
    drag.set_propagation_phase(gtk::PropagationPhase::Capture);
    let origin = Rc::new(std::cell::Cell::new(None::<(f64, f64)>));
    let start = origin.clone();
    let e = editor.clone();
    drag.connect_drag_begin(move |gesture, x, y| {
        let view = e.draft.borrow();
        let column = (x / f64::from(e.grid.width().max(1)) * f64::from(view.columns)).floor() as u8;
        let row = (y / f64::from(e.grid.height().max(1)) * f64::from(view.rows)).floor() as u8;
        let empty = x >= 0.
            && y >= 0.
            && column < view.columns
            && row < view.rows
            && !view.panels.iter().any(|p| {
                column >= p.x && column < p.x + p.width && row >= p.y && row < p.y + p.height
            });
        if empty && !e.preview.get() {
            start.set(Some((x, y)));
        } else {
            start.set(None);
            gesture.set_state(gtk::EventSequenceState::Denied);
        }
    });
    let start = origin.clone();
    let selected = selection.clone();
    let drawing = outline.downgrade();
    let e = editor.clone();
    drag.connect_drag_update(move |gesture, dx, dy| {
        if let Some((sx, sy)) = start.get() {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            let v = e.draft.borrow();
            let cell = |p: f64, axis: i32, count: u8| {
                (p / f64::from(axis.max(1)) * f64::from(count))
                    .floor()
                    .clamp(0., f64::from(count - 1)) as u8
            };
            let x1 = cell(sx.min(sx + dx), e.grid.width(), v.columns);
            let x2 = cell(sx.max(sx + dx), e.grid.width(), v.columns);
            let y1 = cell(sy.min(sy + dy), e.grid.height(), v.rows);
            let y2 = cell(sy.max(sy + dy), e.grid.height(), v.rows);
            selected.set(Some((x1, y1, x2 - x1 + 1, y2 - y1 + 1)));
            if let Some(d) = drawing.upgrade() {
                d.queue_draw();
            }
        }
    });
    let drawing = outline.downgrade();
    let e = editor.clone();
    drag.connect_drag_end(move |_, dx, dy| {
        let region = selection.take();
        origin.set(None);
        if let Some(d) = drawing.upgrade() {
            d.queue_draw();
        }
        if dx.abs() + dy.abs() > 8. {
            if let Some((x, y, w, h)) = region {
                choose_camera(&e, Some((x, y)), (w, h));
            }
        }
    });
    editor.grid.add_controller(drag);
    content.append(&canvas_scroll);
    root.append(&editor.feedback);
    draw(&editor);
    if let Some(error) = conversion_error {
        editor
            .feedback
            .set_text(&format!("Grade original preservada: {error}"));
    }
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
        let mut v = e.draft.borrow().clone();
        match v.dashboard_layout(c, r, main) {
            Ok(()) => {
                e.columns
                    .set_value(if main { 4. } else { f64::from(12 / c) });
                e.rows.set_value(if main { 4. } else { f64::from(12 / r) });
                apply(&e, v);
            }
            Err(err) => {
                e.feedback.set_text(&err.to_string());
                p.set_selected(0);
            }
        }
    });
    let e = editor.clone();
    let layout_choice = preset.clone();
    resize.connect_clicked(move |_| {
        choose_camera(
            &e,
            None,
            (e.columns.value_as_int() as u8, e.rows.value_as_int() as u8),
        );
        layout_choice.set_selected(0);
    });
    let e = editor.clone();
    fill.connect_clicked(move |_| {
        let mut v = e.draft.borrow().clone();
        let mut added = 0;
        for c in e.state.cameras.borrow().iter() {
            if !v.panels.iter().any(|p| p.camera_id == c.id)
                && v.add_camera_sized(
                    &c.id,
                    None,
                    (e.columns.value_as_int() as u8, e.rows.value_as_int() as u8),
                )
                .is_ok()
            {
                added += 1;
            }
        }
        if added > 0 {
            apply(&e, v);
        } else {
            e.feedback
                .set_text("Nenhuma câmera disponível para os espaços livres.");
        }
    });
    let e = editor.clone();
    let layout_choice = preset.clone();
    undo.connect_clicked(move |_| {
        let previous = e.history.borrow_mut().pop();
        if let Some(v) = previous {
            *e.draft.borrow_mut() = v;
            draw(&e);
            layout_choice.set_selected(0);
            e.feedback.set_text("Última alteração de layout desfeita.");
        }
    });
    let e = editor.clone();
    preview.connect_toggled(move |button| {
        let active = button.is_active();
        e.preview.set(active);
        layout.set_sensitive(!active);
        fill.set_sensitive(!active);
        library.set_sensitive(!active);
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
                    if s2.current.borrow().as_ref() != Some(&view.id) {
                        s2.focus.borrow_mut().take();
                    }
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
fn draw(e: &Editor) {
    while let Some(c) = e.grid.first_child() {
        e.grid.remove(&c);
    }
    let view = e.draft.borrow().clone();

    if let Some(undo) = e.undo.upgrade() {
        undo.set_sensitive(!e.history.borrow().is_empty());
    }
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
            b.set_tooltip_text(Some("Clique para escolher uma câmera ou arraste até aqui"));
            b.set_hexpand(true);
            b.set_vexpand(true);
            b.set_size_request(20, 20);
            b.add_css_class("cctv-grid-cell");
            b.set_sensitive(!preview);
            let e2 = e.clone();
            b.update_property(&[gtk::accessible::Property::Label(&format!(
                "Adicionar câmera na coluna {}, linha {}",
                x + 1,
                y + 1
            ))]);
            b.connect_clicked(move |_| {
                choose_camera(
                    &e2,
                    Some((x, y)),
                    (
                        e2.columns.value_as_int() as u8,
                        e2.rows.value_as_int() as u8,
                    ),
                )
            });
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
                let t = tile(&e.state, &c, p.aspect, false, None);
                e.grid
                    .attach(&t, p.x as i32, p.y as i32, p.width as i32, p.height as i32);
            }
            continue;
        }
        let container = gtk::Box::new(gtk::Orientation::Vertical, 4);
        container.add_css_class("cctv-editor-panel");
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
        b.set_has_frame(false);
        if let Some(label) = b.child().and_then(|w| w.downcast::<gtk::Label>().ok()) {
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            label.set_width_chars(1);
        }
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
        handle.set_size_request(24, 24);
        handle.add_css_class("cctv-resize-handle");
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
            let old = e.draft.borrow().clone();
            if old != v {
                let mut h = e.history.borrow_mut();
                if h.len() == 20 {
                    h.remove(0);
                }
                h.push(old);
            }
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
        if e.preview.get() {
            return false;
        }
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
                    width: e.columns.value_as_int() as u8,
                    height: e.rows.value_as_int() as u8,
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
        .transient_for(
            &e.window
                .upgrade()
                .map(|w| w.upcast::<gtk::Window>())
                .unwrap_or_else(|| e.state.window.clone().upcast()),
        )
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
    grid.attach(&forms::label(&dropdown, "Câmera"), 0, 0, 1, 1);
    grid.attach(&dropdown, 1, 0, 1, 1);
    let aspect =
        gtk::DropDown::from_strings(&["Manter proporção", "Preencher e recortar", "Esticar"]);
    aspect.set_selected(match p.aspect {
        Aspect::Fit => 0,
        Aspect::Fill => 1,
        Aspect::Stretch => 2,
    });
    grid.attach(&forms::label(&aspect, "Imagem"), 0, 1, 1, 1);
    grid.attach(&aspect, 1, 1, 1, 1);
    let mut controls = vec![];
    for (i, (label, value, min)) in [
        ("Coluna", p.x + 1, 1.),
        ("Linha", p.y + 1, 1.),
        ("Largura", p.width, 1.),
        ("Altura", p.height, 1.),
    ]
    .into_iter()
    .enumerate()
    {
        let max = if i == 0 || i == 2 {
            view.columns
        } else {
            view.rows
        };
        let spin = gtk::SpinButton::with_range(min, f64::from(max), 1.);
        spin.set_value(value as f64);
        grid.attach(&forms::label(&spin, label), 0, i as i32 + 2, 1, 1);
        grid.attach(&spin, 1, i as i32 + 2, 1, 1);
        controls.push(spin);
    }
    let error = gtk::Label::new(None);
    error.set_wrap(true);
    error.set_max_width_chars(45);
    error.set_xalign(0.);
    error.add_css_class("error");
    grid.attach(&error, 0, 6, 2, 1);
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
                    p.x = (controls[0].value_as_int() - 1) as u8;
                    p.y = (controls[1].value_as_int() - 1) as u8;
                    p.width = controls[2].value_as_int() as u8;
                    p.height = controls[3].value_as_int() as u8;
                    p.aspect = match aspect.selected() {
                        1 => Aspect::Fill,
                        2 => Aspect::Stretch,
                        _ => Aspect::Fit,
                    };
                }
                if let Err(err) = v.validate() {
                    error.set_text(&err.to_string());
                    controls[0].grab_focus();
                    return;
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
    let controllers = grid.observe_controllers();
    let owned: Vec<_> = (0..controllers.n_items())
        .filter_map(|i| controllers.item(i)?.downcast::<gtk::EventController>().ok())
        .collect();
    for controller in owned {
        grid.remove_controller(&controller);
    }
    while let Some(child) = grid.first_child() {
        grid.remove(&child);
    }
}

fn add_camera(e: &Editor, id: &str, position: Option<(u8, u8)>) {
    if e.preview.get() {
        return;
    }
    let mut v = e.draft.borrow().clone();
    match v.add_camera_sized(
        id,
        position,
        (e.columns.value_as_int() as u8, e.rows.value_as_int() as u8),
    ) {
        Ok(()) => {
            apply(e, v);
        }
        Err(err) => e.feedback.set_text(&err.to_string()),
    }
}
fn choose_camera(e: &Editor, position: Option<(u8, u8)>, size: (u8, u8)) {
    if e.preview.get() {
        return;
    }
    let parent = e
        .window
        .upgrade()
        .map(|w| w.upcast::<gtk::Window>())
        .unwrap_or_else(|| e.state.window.clone().upcast());
    let d = gtk::Dialog::builder()
        .transient_for(&parent)
        .destroy_with_parent(true)
        .modal(true)
        .title("Escolher câmera")
        .default_width(380)
        .default_height(300)
        .build();
    d.add_button("Cancelar", gtk::ResponseType::Cancel);
    let list = gtk::Box::new(gtk::Orientation::Vertical, 8);
    margins(&list, 16);
    if e.state.cameras.borrow().is_empty() {
        forms::help(&list,"Nenhuma câmera cadastrada. Cadastre uma câmera na biblioteca antes de adicioná-la à View.");
    }
    for c in e.state.cameras.borrow().iter() {
        let b = gtk::Button::with_label(&c.name);
        let e = e.clone();
        let id = c.id.clone();
        let weak = d.downgrade();
        b.connect_clicked(move |_| {
            let mut v = e.draft.borrow().clone();
            match v.add_camera_sized(&id, position, size) {
                Ok(()) => {
                    apply(&e, v);
                }
                Err(err) => {
                    e.feedback.set_text(&err.to_string());
                }
            }
            if let Some(d) = weak.upgrade() {
                d.close();
            }
        });
        list.append(&b);
    }
    let scroll = gtk::ScrolledWindow::builder()
        .child(&list)
        .vexpand(true)
        .build();
    d.content_area().append(&scroll);
    d.connect_response(|d, _| d.close());
    d.present();
}
