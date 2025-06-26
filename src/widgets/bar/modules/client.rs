use futures_signals::signal::SignalExt;
use gtk4::prelude::*;

use crate::{APP, singletons::hyprland};

fn icon_or(icon_name: Option<&str>) -> Option<&str> {
    if let Some(name) = icon_name {
        let icon_theme = &APP.lock().unwrap().icon_theme;

        if icon_theme.has_icon(name) {
            Some(name)
        } else {
            Some("emote-love")
        }
    } else {
        Some("emote-love")
    }
}

pub fn new() -> gtk4::Box {
    relm4_macros::view! {
        icon = gtk4::Image {
            set_icon_name: icon_or(Some("go-next-symbolic")),
            set_pixel_size: 16,
            set_css_classes: &["bar-client-icon"],
        },

        class = gtk4::Label {
            set_css_classes: &["bar-client-class"],
            set_label: "No active client",
            set_ellipsize: gtk4::pango::EllipsizeMode::End,
            set_max_width_chars: 28,
            set_justify: gtk4::Justification::Left,
            set_xalign: 0.0
        },

        widget = gtk4::Box {
            set_css_classes: &["bar-widget", "bar-client"],
            set_hexpand: false,

            append: &icon,
            append: &class
        }
    }

    // Subscribe to Hyprland signals to update the client information
    let hyprland_future = hyprland::HYPRLAND.active_client.signal_cloned().for_each(move |client| {
        if let Some(client) = client {
            class.set_label(&client.class);
            icon.set_icon_name(icon_or(Some(&client.class.to_lowercase())));
        } else {
            class.set_label("No active client");
            icon.set_icon_name(icon_or(None));
        }

        async {}
    });

    gtk4::glib::MainContext::default().spawn_local(hyprland_future);

    widget
}