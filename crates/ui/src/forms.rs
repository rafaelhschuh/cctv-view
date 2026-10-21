use super::*;
/// Visible labels and persistent accessible names use the same vocabulary.
pub fn label(control: &impl IsA<gtk::Widget>, text: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(text));
    l.set_xalign(0.);
    l.set_mnemonic_widget(Some(control));
    control
        .as_ref()
        .update_property(&[gtk::accessible::Property::Label(text)]);
    l
}
pub fn row(parent: &gtk::Box, text: &str, control: &impl IsA<gtk::Widget>) {
    let row = gtk::Box::new(gtk::Orientation::Vertical, 6);
    row.append(&label(control, text));
    row.append(control);
    parent.append(&row);
}
pub fn entry(parent: &gtk::Box, text: &str, value: &str) -> gtk::Entry {
    let e = gtk::Entry::builder()
        .text(value)
        .hexpand(true)
        .activates_default(true)
        .build();
    e.set_tooltip_text(Some(text));
    row(parent, text, &e);
    e
}
pub fn section(parent: &gtk::Box, text: &str) {
    let l = gtk::Label::new(Some(text));
    l.set_xalign(0.);
    l.add_css_class("heading");
    l.set_margin_top(8);
    parent.append(&l);
}
pub fn page(notebook: &gtk::Notebook, title: &str) -> gtk::Box {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    margins(&content, 20);
    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&content)
        .vexpand(true)
        .build();
    notebook.append_page(&scroll, Some(&gtk::Label::new(Some(title))));
    content
}
pub fn help(parent: &gtk::Box, text: &str) {
    let l = gtk::Label::new(Some(text));
    l.set_xalign(0.);
    l.set_wrap(true);
    l.set_max_width_chars(60);
    l.add_css_class("dim-label");
    parent.append(&l);
}

/// Release an entry's keyboard focus before hiding a transient dialog.
pub(crate) fn close(dialog: &gtk::Dialog) {
    gtk::prelude::GtkWindowExt::set_focus(dialog, None::<&gtk::Widget>);
    dialog.close();
}

/// Empty feedback must not reserve a line or an extra gap above dialog actions.
pub(crate) fn feedback(parent: &gtk::Box) -> gtk::Label {
    let label = gtk::Label::new(None);
    label.set_xalign(0.);
    label.set_wrap(true);
    label.set_visible(false);
    label.connect_label_notify(|label| label.set_visible(!label.text().is_empty()));
    parent.append(&label);
    label
}
