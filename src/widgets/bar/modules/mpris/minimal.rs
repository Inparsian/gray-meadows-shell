use std::cell::RefCell;
use gtk4::prelude::*;

use crate::singletons::mpris;

const ALBUM_ART_WIDTH: i32 = 24; // Expected width of the album art image
const ALBUM_ART_HEIGHT: i32 = 24; // Expected height of the album art image
const WIDGET_WIDTH: i32 = 175;
const MAX_TRACK_WIDTH: i32 = WIDGET_WIDTH - ALBUM_ART_WIDTH;

fn get_mpris_player_label_text() -> String {
    mpris::get_default_player()
        .map_or_else(|| "No players".to_owned(), |player| player.metadata.title.unwrap_or("No title".to_owned()))
}

pub fn minimal() -> gtk4::Box {
    let current_art_url = RefCell::new(" ".to_owned());

    view! {
        no_players_widget = gtk4::Label {
            set_css_classes: &["bar-mpris-track"],
            set_label: "No players",
            set_hexpand: true,
            set_xalign: 0.5
        },

        current_track = gtk4::Label {
            set_css_classes: &["bar-mpris-track"],
            set_label: &get_mpris_player_label_text(),
            set_hexpand: true,
            set_xalign: 0.5,
            set_ellipsize: gtk4::pango::EllipsizeMode::End
        },

        current_album_art = gtk4::Image {
            set_width_request: ALBUM_ART_WIDTH,
            set_height_request: ALBUM_ART_HEIGHT,
            set_pixel_size: ALBUM_ART_WIDTH,
        },

        paused_overlay = gtk4::CenterBox {
            set_css_classes: &["bar-mpris-paused-overlay"],
            set_center_widget: Some(&gtk4::Label::new(Some("▶"))),
            set_visible: mpris::get_default_player().is_some_and(|p| p.playback_status != mpris::mpris_player::PlaybackStatus::Playing),
        },

        album_overlay = gtk4::Overlay {
            set_child: Some(&current_album_art),
            add_overlay: &paused_overlay,
        },

        players_widget = gtk4::Box {
            set_hexpand: false,

            append: &album_overlay,

            libadwaita::Clamp {
                set_child: Some(&current_track),
                set_width_request: MAX_TRACK_WIDTH,
                set_maximum_size: MAX_TRACK_WIDTH,
                set_unit: libadwaita::LengthUnit::Px
            }
        },

        widget = gtk4::Box {
            set_hexpand: false,
            set_width_request: WIDGET_WIDTH,

            append: &no_players_widget,
            append: &players_widget
        },
    }

    mpris::subscribe_to_default_player_changes(move |_| {
        if let Some(default_player) = mpris::get_default_player() {
            no_players_widget.hide();
            players_widget.show();

            current_track.set_label(&get_mpris_player_label_text());
            paused_overlay.set_visible(default_player.playback_status != mpris::mpris_player::PlaybackStatus::Playing);

            let make_blank_art = || {
                // Create blank pixbuf filled with color #0D0D0D
                let blank_pixbuf = gtk4::gdk_pixbuf::Pixbuf::new(
                    gtk4::gdk_pixbuf::Colorspace::Rgb,
                    true,
                    8,
                    ALBUM_ART_WIDTH,
                    ALBUM_ART_HEIGHT
                );

                if let Some(blank_pixbuf) = blank_pixbuf {
                    blank_pixbuf.fill(0x0D0D_0DFF);
                    current_album_art.set_from_pixbuf(Some(&blank_pixbuf));
                } else {
                    eprintln!("Failed to create blank pixbuf for album art! :O");
                }
            };

            if let Some(art_url) = default_player.metadata.art_url {
                if *current_art_url.borrow() != *art_url {
                    *current_art_url.borrow_mut() = art_url.clone();

                    // URL-decode the album art URL
                    let art_url = match urlencoding::decode(&art_url.replace("file://", "")) {
                        Ok(decoded) => decoded.into_owned(),
                        Err(e) => {
                            eprintln!("Failed to decode album art URL: {}", e);
                            return;
                        }
                    };

                    // Make pixbuf from album art
                    match gtk4::gdk_pixbuf::Pixbuf::from_file(&*art_url) {
                        Ok(pixbuf) => {
                            if let Some(scaled_pixbuf) = pixbuf.scale_simple(
                                ALBUM_ART_WIDTH, 
                                ALBUM_ART_HEIGHT, 
                                gtk4::gdk_pixbuf::InterpType::Tiles
                            ) {
                                scaled_pixbuf.saturate_and_pixelate(
                                    &scaled_pixbuf,
                                    0.0,
                                    false
                                );

                                current_album_art.set_from_pixbuf(Some(&scaled_pixbuf));
                            } else {
                                eprintln!("Failed to scale album art from file: {}", art_url);
                                make_blank_art();
                            }
                        },

                        Err(e) => {
                            eprintln!("Failed to load album art from file: {}: {}", art_url, e);
                            make_blank_art();
                        }
                    }
                }
            } else if !(*current_art_url.borrow()).is_empty() {
                current_art_url.borrow_mut().clear();
                make_blank_art();
            }
        } else {
            players_widget.hide();
            no_players_widget.show();
        }
    });

    widget
}