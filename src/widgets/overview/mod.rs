use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use crate::{ipc, singletons::hyprland};

pub fn new(application: &libadwaita::Application) {
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

    ipc::listen_for_messages_local(move |message| {
        if message.as_str() == "toggle_overview" {
            let monitor = hyprland::get_active_monitor();

            if window.is_visible() {
                window.hide();
            } else {
                window.set_monitor(monitor.as_ref());
                window.show();
            }
        }
    });
}