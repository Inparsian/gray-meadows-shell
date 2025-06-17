use gtk4::prelude::*;

use crate::singletons::mpris;

pub fn new() -> gtk4::Box {
    relm4_macros::view! {
        current_track = gtk4::Label {
            set_label: &mpris::get_default_player()
                .map_or_else(|| "No player".to_string(), |player| player.metadata.title.unwrap_or_default().to_string()),
        },

        widget = gtk4::Box {
            set_css_classes: &["bar-widget"],
            set_hexpand: false,

            append: &current_track,
        }
    }

    mpris::subscribe_to_default_player_changes(move |_| {
        current_track.set_label(&mpris::get_default_player()
            .map_or_else(|| "No player".to_string(), |player| player.metadata.title.unwrap_or_default().to_string()));
    });

    widget
}