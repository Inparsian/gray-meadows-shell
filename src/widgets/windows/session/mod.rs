use gtk4::prelude::*;

use crate::{widgets::windows::{self, types::fullscreen::FullscreenWindow}};

pub fn session_button(icon: &str, command: &str) -> gtk4::Button {
    let icon = icon.to_owned();
    let command = command.to_owned();

    let button = gtk4::Button::new();
    button.set_valign(gtk4::Align::Center);
    button.set_css_classes(&["session-button"]);
    button.connect_clicked(move |_| {
        windows::hide("session");

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

pub fn new(application: &libadwaita::Application) -> FullscreenWindow {
    let lock_button = session_button("lock", "loginctl lock-session");
    let logout_button = session_button("logout", "pkill Hyprland || loginctl terminate-user $USER");
    let suspend_button = session_button("remove_circle_outline", "systemctl suspend || loginctl suspend");
    let hibernate_button = session_button("mode_standby", "systemctl hibernate || loginctl hibernate");
    let reboot_button = session_button("restart_alt", "systemctl reboot || loginctl reboot");
    let shutdown_button = session_button("power_settings_new", "systemctl poweroff || loginctl poweroff");

    view! {
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
        }
    };

    FullscreenWindow::new(
        application,
        &["session-window"],
        &session_box,
    )
}