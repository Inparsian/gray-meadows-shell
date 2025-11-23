mod header;
mod quicktoggle;

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::{helpers::gesture, ipc, singletons::hyprland};

pub fn new(application: &libadwaita::Application) {
    let header = header::new();

    view! {
        quick_toggles = gtk4::Box {
            set_css_classes: &["sidebar-right-quicktoggles"],
            set_spacing: 4,
            set_orientation: gtk4::Orientation::Horizontal,
            set_hexpand: true,
            set_vexpand: false,

            append: &quicktoggle::keybinds::new(),
            append: &quicktoggle::gamemode::new(),
        },

        right_sidebar_box = gtk4::Box {
            set_css_classes: &["right-sidebar-box"],
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 12,
            set_hexpand: true,
            set_vexpand: true,

            append: &header,
            append: &quick_toggles
        },

        window = gtk4::ApplicationWindow {
            set_css_classes: &["right-sidebar-window"],
            set_application: Some(application),
            init_layer_shell: (),
            set_namespace: Some("right-sidebar"),
            set_monitor: hyprland::get_active_monitor().as_ref(),
            set_keyboard_mode: KeyboardMode::OnDemand,
            set_layer: Layer::Overlay,
            set_anchor: (Edge::Left, false),
            set_anchor: (Edge::Right, true),
            set_anchor: (Edge::Top, true),
            set_anchor: (Edge::Bottom, true),

            set_child: Some(&right_sidebar_box)
        }
    };

    window.add_controller(gesture::on_primary_full_press({
        let window = window.clone();

        move |_, p_xy, r_xy| {
            let allocation = right_sidebar_box.allocation();
            let (px, py) = (p_xy.0 as i32, p_xy.1 as i32);
            let (rx, ry) = (r_xy.0 as i32, r_xy.1 as i32);

            if window.is_visible() && !allocation.contains_point(px, py) && !allocation.contains_point(rx, ry) {
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
        if message.as_str() == "toggle_right_sidebar" {
            let monitor = hyprland::get_active_monitor();

            if window.is_visible() {
                window.hide();
            } else {
                window.set_monitor(monitor.as_ref());
                window.show();
            }
        }

        else if message.as_str() == "hide_right_sidebar" {
            window.hide();
        }
    });
}