use gtk4::prelude::*;

use crate::session::SessionAction;
use super::super::windows::{self, fullscreen::FullscreenWindow};

pub fn session_button(icon: &str, action: SessionAction) -> gtk4::Button {
    let icon = icon.to_owned();

    let button = gtk4::Button::new();
    button.set_valign(gtk4::Align::Center);
    button.set_css_classes(&["session-button"]);
    button.connect_clicked(move |_| {
        windows::hide("session");
        action.run();
    });

    let label = gtk4::Label::new(Some(&icon));
    label.set_css_classes(&["session-button-icon"]);
    button.set_child(Some(&label));

    button
}

pub fn new(application: &libadwaita::Application) -> FullscreenWindow {
    let lock_button = session_button("lock", SessionAction::Lock);
    let logout_button = session_button("logout", SessionAction::Logout);
    let suspend_button = session_button("remove_circle_outline", SessionAction::Suspend);
    let hibernate_button = session_button("mode_standby", SessionAction::Hibernate);
    let reboot_button = session_button("restart_alt", SessionAction::Reboot);
    let shutdown_button = session_button("power_settings_new", SessionAction::Shutdown);

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