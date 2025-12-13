use gtk4::prelude::*;

pub fn extended() -> gtk4::Box {
    view! {
        widget = gtk4::Box {
            set_css_classes: &["bar-sysstats-extended"],
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 0,
        },
    }

    widget
}