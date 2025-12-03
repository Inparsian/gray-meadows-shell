use gtk4::prelude::*;

pub fn extended() -> gtk4::Box {
    view! {
        widget = gtk4::Box {
            set_css_classes: &["bar-mpris-extended"],
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 4,

            gtk4::Label {
                set_label: "Extended MPRIS Module!!!!!!!!!!",
                set_hexpand: true,
                set_xalign: 0.5
            },

            gtk4::Label {
                set_label: "More features coming soon...",
                set_hexpand: true,
                set_xalign: 0.5
            },

            gtk4::Button {
                set_label: "Test Button",
                set_hexpand: true,
                connect_clicked => |_| {
                    println!("Test Button Clicked!");
                }
            }
        },
    }

    widget
}