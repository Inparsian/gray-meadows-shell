use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::{helpers::gesture, ipc, singletons::hyprland};

pub fn session_button(icon: &str, command: &str) -> gtk4::Button {
    let icon = icon.to_owned();
    let command = command.to_owned();

    let button = gtk4::Button::new();
    button.set_valign(gtk4::Align::Center);
    button.set_css_classes(&["session-button"]);
    button.connect_clicked(move |_| {
        ipc::client::send_message("hide_session")
            .expect("Failed to send hide_session message");

        std::process::Command::new("bash")
            .arg("-c")
            .arg(&command)
            .output()
            .expect("Failed to execute command");
    });

    let label = gtk4::Label::new(Some(&icon));
    label.set_css_classes(&["session-button-icon"]);
    button.set_child(Some(&label));

    button
}

pub fn new(application: &libadwaita::Application) {
    let lock_button = session_button("lock", "loginctl lock-session");
    let logout_button = session_button("logout", "pkill Hyprland || loginctl terminate-user $USER");
    let suspend_button = session_button("remove_circle_outline", "systemctl suspend || loginctl suspend");
    let hibernate_button = session_button("mode_standby", "systemctl hibernate || loginctl hibernate");
    let reboot_button = session_button("restart_alt", "systemctl reboot || loginctl reboot");
    let shutdown_button = session_button("power_settings_new", "systemctl poweroff || loginctl poweroff");

    relm4_macros::view! {
        session_box = gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 12,
            set_halign: gtk4::Align::Center,
            set_valign: gtk4::Align::Center,
            set_hexpand: true,

            // First row
            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_css_classes: &["session-box-row1"],
                set_spacing: 12,

                append: &lock_button,
                append: &logout_button,
                append: &suspend_button
            },

            // Second row
            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_css_classes: &["session-box-row2"],
                set_spacing: 12,

                append: &hibernate_button,
                append: &reboot_button,
                append: &shutdown_button
            }
        },

        window = gtk4::ApplicationWindow {
            set_css_classes: &["session-window"],
            set_application: Some(application),
            init_layer_shell: (),
            set_monitor: hyprland::get_active_monitor().as_ref(),
            set_keyboard_mode: KeyboardMode::OnDemand,
            set_layer: Layer::Top,
            set_anchor: (Edge::Left, true),
            set_anchor: (Edge::Right, true),
            set_anchor: (Edge::Top, true),
            set_anchor: (Edge::Bottom, true),

            set_child: Some(&session_box)
        }
    };

    window.add_controller(gesture::on_primary_click({
        let window = window.clone();

        move |_, x, y| {
            if window.is_visible() && !session_box.allocation().contains_point(x as i32, y as i32) {
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
        if message.as_str() == "toggle_session" {
            let monitor = hyprland::get_active_monitor();

            if window.is_visible() {
                window.hide();
            } else {
                window.set_monitor(monitor.as_ref());
                window.show();
            }
        }

        else if message.as_str() == "hide_session" {
            window.hide();
        }
    });
}