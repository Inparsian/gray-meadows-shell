use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use crate::singletons::hyprland;

pub struct Overview {
    pub window: gtk4::ApplicationWindow
}

impl Overview {
    pub fn new(application: &libadwaita::Application) -> Self {
        relm4_macros::view! {
            window = gtk4::ApplicationWindow {
                set_css_classes: &["overview-window"],
                set_application: Some(application),
                init_layer_shell: (),
                set_monitor: hyprland::get_active_monitor().as_ref(),
                set_layer: Layer::Top,
                set_anchor: (Edge::Left, true),
                set_anchor: (Edge::Right, true),
                set_anchor: (Edge::Top, true),
                set_anchor: (Edge::Bottom, true),

                gtk4::Box {
                    set_orientation: gtk4::Orientation::Vertical,
                    set_css_classes: &["overview"],
                    set_spacing: 0,

                    // Placeholder for overview content
                    gtk4::Label {
                        set_label: "Overview content goes here",
                        set_css_classes: &["overview-content"]
                    }
                }
            }
        };

        Self { window }
    }
}