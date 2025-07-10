use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::{helpers::gesture, ipc, singletons::hyprland};

pub fn new(application: &libadwaita::Application) {
    relm4_macros::view! {
        overview_box = gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_css_classes: &["overview"],
            set_spacing: 0,

            // Placeholder for overview content
            gtk4::Label {
                set_label: "Overview content goes here",
                set_css_classes: &["overview-content"]
            }
        },

        window = gtk4::ApplicationWindow {
            set_css_classes: &["overview-window"],
            set_application: Some(application),
            init_layer_shell: (),
            set_monitor: hyprland::get_active_monitor().as_ref(),
            set_keyboard_mode: KeyboardMode::OnDemand,
            set_layer: Layer::Top,
            set_anchor: (Edge::Left, true),
            set_anchor: (Edge::Right, true),
            set_anchor: (Edge::Top, true),
            set_anchor: (Edge::Bottom, true),

            set_child: Some(&overview_box),
        }
    };

    window.add_controller(gesture::on_primary_click({
        let window = window.clone();
        let overview_box = overview_box.clone();

        move |_, x, y| {
            if window.is_visible() && !overview_box.allocation().contains_point(x as i32, y as i32) {
                window.hide();
            }
        }
    }));

    window.add_controller(gesture::on_key_press({
        let window = window.clone();

        move |val, _| {
            if val.name() == Some("Escape".into()) {
                window.hide();
            }
        }
    }));

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