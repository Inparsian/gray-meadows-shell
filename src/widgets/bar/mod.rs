mod modules {
    pub mod clock;
}

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};

pub struct Bar {
    pub window: gtk4::ApplicationWindow,
}

impl Bar {
    pub fn new(application: &gtk4::Application, monitor: &gdk4::Monitor) -> Self {
        relm4_macros::view! {
            left_box = gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 1
            },

            center_box = gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 1,

                append: &modules::clock::new()
            },

            right_box = gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 1
            },

            window = gtk4::ApplicationWindow {
                set_application: Some(application),
                init_layer_shell: (),
                set_monitor: Some(&monitor),
                set_default_height: 33,
                set_layer: Layer::Top,
                set_anchor: (Edge::Left, true),
                set_anchor: (Edge::Right, true),
                set_anchor: (Edge::Top, true),
                auto_exclusive_zone_enable: (),

                gtk4::CenterBox {
                    set_css_classes: &["bar"],

                    // Left side widgets
                    set_start_widget: Some(&left_box),

                    // Center widgets
                    set_center_widget: Some(&center_box),

                    // Right side widgets
                    set_end_widget: Some(&right_box),
                }
            }
        }

        Bar {
            window
        }
    }
}