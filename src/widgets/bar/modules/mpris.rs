use gtk4::prelude::*;
use internment::Intern;
use once_cell::sync::Lazy;

use crate::singletons::mpris;

static NO_ARTIST: Lazy<Intern<Vec<String>>> = Lazy::new(|| Intern::new(vec!["No artist".to_string()]));
static NO_TITLE: Lazy<Intern<String>> = Lazy::new(|| Intern::new("No title".to_string()));

fn get_mpris_player_label_text() -> String {
    mpris::get_default_player()
        .map_or_else(|| "No MPRIS players".to_string(), |player| format!("{} - {}",
            player.metadata.artist.unwrap_or(NO_ARTIST.clone()).join(", "),
            player.metadata.title.unwrap_or(NO_TITLE.clone()).to_string()
        ))
}

pub fn new() -> gtk4::Box {
    relm4_macros::view! {
        current_track = gtk4::Label {
            set_label: &get_mpris_player_label_text(),
            set_hexpand: true,
            set_xalign: 0.5
        },

        widget = gtk4::Box {
            set_css_classes: &["bar-widget", "bar-mpris"],
            set_hexpand: false,

            append: &current_track,
        }
    }

    mpris::subscribe_to_default_player_changes(move || {
        current_track.set_label(&get_mpris_player_label_text());
    });

    widget
}