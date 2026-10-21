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
