use gtk4::prelude::*;

pub fn new() -> gtk4::Box {
    relm4_macros::view! {
        widget = gtk4::Box {
            set_css_classes: &["dot-separator"],
            set_valign: gtk4::Align::Center,
            set_hexpand: false
        }
    }

    widget
}