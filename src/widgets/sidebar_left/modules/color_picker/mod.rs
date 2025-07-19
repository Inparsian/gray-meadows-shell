use gtk4::prelude::*;

pub fn new() -> gtk4::Box {
    view! {
        widget = gtk4::Box {
            set_css_classes: &["ColorPicker"],
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 12,
            set_hexpand: true,
            set_vexpand: true,

            gtk4::Label {
                set_text: "Color Picker"
            }
        }
    }

    widget
}