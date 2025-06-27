use futures_signals::signal::{Mutable, SignalExt};
use gtk4::prelude::*;

use crate::{APP, singletons::hyprland};

const MAX_CLASS_WIDTH: i32 = 29;
const MAX_TITLE_WIDTH: i32 = 54;

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
    let reveal_title = Mutable::new(false);

    relm4_macros::view! {
        reveal_title_gesture = gtk4::EventControllerMotion {
            connect_enter: {
                let reveal_title = reveal_title.clone();
                move |_, _, _| {
                    reveal_title.set(true);
                }
            },

            connect_leave: {
                let reveal_title = reveal_title.clone();
                move |_| {
                    reveal_title.set(false);
                }
            },
        },

        icon = gtk4::Image {
            set_icon_name: icon_or(Some("go-next-symbolic")),
            set_pixel_size: 16,
            set_css_classes: &["bar-client-icon"],
        },

        class = gtk4::Label {
            set_css_classes: &["bar-client-class"],
            set_label: "No active client",
            set_ellipsize: gtk4::pango::EllipsizeMode::End,
            set_max_width_chars: MAX_CLASS_WIDTH,
            set_justify: gtk4::Justification::Left,
            set_hexpand: true,
            set_xalign: 0.0
        },

        title = gtk4::Label {
            set_css_classes: &["bar-client-title"],
            set_label: "No active client",
            set_ellipsize: gtk4::pango::EllipsizeMode::End,
            set_max_width_chars: MAX_TITLE_WIDTH,
            set_justify: gtk4::Justification::Left,
            set_hexpand: true,
            set_xalign: 0.0
        },

        class_revealer = gtk4::Revealer {
            set_transition_type: gtk4::RevealerTransitionType::SlideRight,
            set_transition_duration: 175,
            set_reveal_child: true,
            set_child: Some(&class)
        },

        title_revealer = gtk4::Revealer {
            set_transition_type: gtk4::RevealerTransitionType::SlideRight,
            set_transition_duration: 175,
            set_reveal_child: false,
            set_child: Some(&title)
        },

        widget = gtk4::Box {
            set_css_classes: &["bar-widget", "bar-client"],
            set_hexpand: false,

            add_controller: reveal_title_gesture,

            append: &icon,
            append: &class_revealer,
            append: &title_revealer
        }
    }

    // Subscribe to Hyprland signals to update the client information
    let hyprland_future = hyprland::HYPRLAND.active_client.signal_cloned().for_each(move |client| {
        if let Some(client) = client {
            class.set_label(&client.class);
            title.set_label(&client.title);
            icon.set_icon_name(icon_or(Some(&client.class.to_lowercase())));

            // I hate GTK4
            class.set_ellipsize(if client.class.len() < MAX_CLASS_WIDTH as usize {
                gtk4::pango::EllipsizeMode::None
            } else {
                gtk4::pango::EllipsizeMode::End
            });
            
            title.set_ellipsize(if client.title.len() < MAX_TITLE_WIDTH as usize {
                gtk4::pango::EllipsizeMode::None
            } else {
                gtk4::pango::EllipsizeMode::End
            });
        } else {
            class.set_label("No active client");
            icon.set_icon_name(icon_or(None));
        }

        async {}
    });

    let reveal_title_future = reveal_title.signal().for_each(move |reveal| {
        if reveal {
            title_revealer.set_reveal_child(true);
            class_revealer.set_reveal_child(false);
        } else {
            title_revealer.set_reveal_child(false);
            class_revealer.set_reveal_child(true);
        }
        async {}
    });

    gtk4::glib::MainContext::default().spawn_local(hyprland_future);
    gtk4::glib::MainContext::default().spawn_local(reveal_title_future);

    widget
}