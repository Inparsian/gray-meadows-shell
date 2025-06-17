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

        widget_middle_click_gesture = &gtk4::GestureClick::new() {
            set_button: gdk4::ffi::GDK_BUTTON_MIDDLE.try_into().unwrap(), // ?????
            connect_pressed: |_, _, _, _| {
                if let Some(player) = mpris::get_default_player() {
                    if let Err(e) = player.play_pause() {
                        eprintln!("Failed to toggle play/pause: {}", e);
                    }
                } else {
                    eprintln!("No MPRIS player available to toggle play/pause.");
                }
            }
        },

        widget_right_click_gesture = &gtk4::GestureClick::new() {
            set_button: gdk4::ffi::GDK_BUTTON_SECONDARY.try_into().unwrap(), // ?????
            connect_pressed: |_, _, _, _| {
                if let Some(player) = mpris::get_default_player() {
                    if let Err(e) = player.next() {
                        eprintln!("Failed to skip to next track: {}", e);
                    }
                } else {
                    eprintln!("No MPRIS player available to skip to next track.");
                }
            }
        },

        widget = gtk4::Box {
            set_css_classes: &["bar-widget", "bar-mpris"],
            set_hexpand: false,

            add_controller: widget_middle_click_gesture,
            add_controller: widget_right_click_gesture,

            append: &current_track,
        }
    }

    mpris::subscribe_to_default_player_changes(move || {
        current_track.set_label(&get_mpris_player_label_text());
    });

    widget
}