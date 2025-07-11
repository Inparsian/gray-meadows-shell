mod item;
mod modules;

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::{helpers::gesture, ipc, singletons::hyprland};

pub fn new(application: &libadwaita::Application) {
    relm4_macros::view! {
        entry_prompt_revealer = gtk4::Revealer {
            set_transition_type: gtk4::RevealerTransitionType::Crossfade,
            set_reveal_child: true,

            gtk4::Label {
                set_css_classes: &["entry-prompt-label"],
                set_label: "Type to search, and stuff",
            }
        },

        entry = gtk4::Entry {
            set_css_classes: &["entry-prompt"],
            set_has_frame: false,

            connect_activate: move |entry| {
                let text = entry.text().to_string();
                if !text.is_empty() {
                    println!("Search query: {}", text);
                }
            },

            connect_changed: {
                let entry_prompt_revealer = entry_prompt_revealer.clone();
                move |entry| {
                    if !entry.text().is_empty() {
                        entry_prompt_revealer.set_reveal_child(false);
                        entry.style_context().add_class("entry-extended");
                    } else {
                        entry_prompt_revealer.set_reveal_child(true);
                        entry.style_context().remove_class("entry-extended");
                    }
                }
            }
        },

        entry_box = gtk4::Box {
            set_css_classes: &["entry-box"],
            set_orientation: gtk4::Orientation::Horizontal,
            set_hexpand: true,

            append: &entry,
        },

        overview_box = gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 8,
            set_halign: gtk4::Align::Center,
            set_valign: gtk4::Align::Center,
            set_hexpand: true,

            gtk4::Overlay {
                set_child: Some(&entry_box),
                add_overlay: &entry_prompt_revealer,
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
        let entry = entry.clone();
        let overview_box = overview_box.clone();

        move |_, x, y| {
            if window.is_visible() && !overview_box.allocation().contains_point(x as i32, y as i32) {
                window.hide();
                entry.set_text("");
            }
        }
    }));

    window.add_controller(gesture::on_key_press({
        let window = window.clone();
        let entry = entry.clone();

        move |val, _| {
            if val.name() == Some("Escape".into()) {
                window.hide();
                entry.set_text("");
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

        else if message.as_str() == "hide_overview" {
            window.hide();
        }
    });
}