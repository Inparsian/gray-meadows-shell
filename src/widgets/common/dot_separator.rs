use gtk::prelude::*;

pub fn new() -> gtk::Box {
    view! {
        widget = gtk::Box {
            set_css_classes: &["dot-separator"],
            set_valign: gtk::Align::Center,
            set_hexpand: false
        }
    }

    widget
}