use gtk4::prelude::*;

pub fn new() -> gtk4::Box {
    let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    root.set_css_classes(&["calendar-tab-root"]);

    let coming_soon_label = gtk4::Label::new(Some("Calendar coming soon!"));
    coming_soon_label.set_css_classes(&["calendar-tab-coming-soon"]);
    coming_soon_label.set_hexpand(true);
    coming_soon_label.set_vexpand(true);
    root.append(&coming_soon_label);

    root
}