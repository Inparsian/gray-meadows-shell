use futures_signals::signal::{Mutable, SignalExt};
use gtk4::prelude::*;

use crate::{
    singletons::hyprland,
    widgets::bar::wrapper::BarModuleWrapper,
    APP
};

const MAX_CLASS_WIDTH: i32 = 29;
const MAX_TITLE_WIDTH: i32 = 54;

fn icon_or(icon_name: Option<&str>) -> Option<&str> {
    if let Some(name) = icon_name {
        let icon_theme = &APP.lock().unwrap().icon_theme;

        if icon_theme.has_icon(name) {
            return Some(name);
        }
    }
    
    Some("emote-love")
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
            set_icon_name: Some("emote-love"),
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

        client_box = gtk4::Box {
            set_orientation: gtk4::Orientation::Horizontal,
            set_spacing: 0,

            append: &icon,
            append: &title_revealer,
            append: &class_revealer
        },

        workspace_label = gtk4::Label {
            set_css_classes: &["bar-client-workspace"],
            set_label: "Workspace 1",
            set_xalign: 0.0,
            set_hexpand: false
        },

        widget = gtk4::Box {
            set_css_classes: &["bar-widget", "bar-client"],
            set_hexpand: false,

            append: &client_box,
            append: &workspace_label
        }
    }

    // Subscribe to Hyprland signals to update the client information
    let hyprland_active_client_future = hyprland::HYPRLAND.active_client.signal_cloned().for_each(move |client| {
        if let Some(client) = client {
            client_box.set_visible(true);
            workspace_label.set_visible(false);

            class.set_label(&client.class);
            title.set_label(&client.title);
            icon.set_icon_name(icon_or(Some(&client.class.to_lowercase())));

            // I hate GTK4
            let get_ellipsize = |s: String, max_len: i32| if s.chars().count() as i32 <= max_len {
                gtk4::pango::EllipsizeMode::None
            } else {
                gtk4::pango::EllipsizeMode::End
            };

            class.set_ellipsize(get_ellipsize(client.class, MAX_CLASS_WIDTH));
            title.set_ellipsize(get_ellipsize(client.title, MAX_TITLE_WIDTH));
        } else {
            let active_workspace = hyprland::HYPRLAND.active_workspace.get_cloned();

            client_box.set_visible(false);
            workspace_label.set_visible(true);
            
            workspace_label.set_label(&active_workspace.map_or(
                "No active workspace".to_owned(), 
                |w| format!("Workspace {}", w.id)
            ));
        }

        async {}
    });

    let reveal_title_future = reveal_title.signal().for_each(move |reveal| {
        title_revealer.set_reveal_child(reveal);
        class_revealer.set_reveal_child(!reveal);
        
        async {}
    });

    gtk4::glib::MainContext::default().spawn_local(hyprland_active_client_future);
    gtk4::glib::MainContext::default().spawn_local(reveal_title_future);

    BarModuleWrapper::new(widget)
        .add_controller(reveal_title_gesture)
        .get_widget()
}