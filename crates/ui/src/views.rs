use super::*;
/// Commands belong to their View; they never rely on an unrelated current selection.
pub fn menu(s: &Shared, v: &View, index: usize) -> gtk::MenuButton {
    let menu = gtk::MenuButton::builder()
        .icon_name("view-more-symbolic")
        .tooltip_text(format!("Ações de {}", v.name))
        .build();
    menu.update_property(&[gtk::accessible::Property::Label(&format!(
        "Ações da View {}",
        v.name
    ))]);
    let popover = gtk::Popover::new();
    let list = gtk::Box::new(gtk::Orientation::Vertical, 4);
    margins(&list, 6);
    popover.set_child(Some(&list));
    menu.set_popover(Some(&popover));
    let initial = s.preferences.borrow().initial_view.as_deref() == Some(&v.id);
    for (label, action) in [
        ("Editar / renomear", 0),
        ("Duplicar", 1),
        (
            if v.favorite {
                "Remover dos favoritos"
            } else {
                "Marcar como favorita"
            },
            2,
        ),
        (
            if initial {
                "Usar última View ao iniciar"
            } else {
                "Abrir esta View ao iniciar"
            },
            3,
        ),
        ("Mover para cima", 4),
        ("Mover para baixo", 5),
        ("Excluir View…", 6),
    ] {
        let b = gtk::Button::with_label(label);
        list.append(&b);
        if action == 4 {
            b.set_sensitive(index > 0);
        }
        if action == 5 {
            b.set_sensitive(index + 1 < s.views.borrow().len());
        }
        if action == 6 {
            b.add_css_class("destructive-action");
        }
        let s = s.clone();
        let v = v.clone();
        let weak = popover.downgrade();
        b.connect_clicked(move |_| {
            if let Some(p) = weak.upgrade() {
                p.popdown();
            }
            match action {
                0 => editor::open(&s, v.clone()),
                1 => {
                    let mut copy = v.duplicate();
                    copy.order = s.views.borrow().len() as i32;
                    persist_view(&s, copy, true);
                }
                2 => {
                    let mut view = v.clone();
                    view.favorite = !view.favorite;
                    persist_view(&s, view, false);
                }
                3 => {
                    s.preferences.borrow_mut().initial_view =
                        if initial { None } else { Some(v.id.clone()) };
                    save_preferences(&s);
                    refresh_sidebar(&s);
                }
                4 => reorder(&s, &v.id, -1),
                5 => reorder(&s, &v.id, 1),
                _ => delete(&s, &v),
            }
        });
    }
    menu
}
fn reorder(s: &Shared, id: &str, delta: i32) {
    let mut views = s.views.borrow().clone();
    let Some(index) = views.iter().position(|v| v.id == id) else {
        return;
    };
    let next = index as i32 + delta;
    if next < 0 || next >= views.len() as i32 {
        return;
    }
    views.swap(index, next as usize);
    let ids: Vec<_> = views.iter().map(|v| v.id.clone()).collect();
    let path = s.path.clone();
    let s = s.clone();
    jobs::run(
        move || -> Result<_> {
            let mut store = Store::open(path)?;
            store.reorder_views(&ids)?;
            store.views()
        },
        move |result| match result {
            Ok(views) => {
                *s.views.borrow_mut() = views;
                refresh_sidebar(&s);
            }
            Err(e) => message(&s.window, &e.to_string()),
        },
    );
}
fn delete(s: &Shared, v: &View) {
    let s2 = s.clone();
    let v = v.clone();
    confirm(
        &s.window,
        &format!(
            "Excluir a View “{}”? As câmeras cadastradas serão mantidas.",
            v.name
        ),
        move || {
            let path = s2.path.clone();
            let id = v.id.clone();
            let s = s2.clone();
            jobs::run(
                move || Store::open(path)?.delete_view(&id),
                move |result: Result<()>| match result {
                    Ok(()) => {
                        s.views.borrow_mut().retain(|view| view.id != v.id);
                        let was_current = s.current.borrow().as_ref() == Some(&v.id);
                        if was_current {
                            *s.current.borrow_mut() =
                                s.views.borrow().first().map(|v| v.id.clone());
                            s.focus.borrow_mut().take();
                        }
                        let was_initial =
                            s.preferences.borrow().initial_view.as_ref() == Some(&v.id);
                        if was_initial {
                            s.preferences.borrow_mut().initial_view = None;
                            save_preferences(&s);
                        }
                        refresh(&s);
                    }
                    Err(e) => message(&s.window, &e.to_string()),
                },
            );
        },
    );
}
