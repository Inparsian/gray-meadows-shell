use std::cell::RefCell;
use gtk4::prelude::*;

use crate::{
    helpers::gesture,
    singletons::mpris,
    widgets::bar::wrapper::BarModuleWrapper
};

const VOLUME_STEP: f64 = 0.05;
const ALBUM_ART_WIDTH: i32 = 23; // Expected width of the album art image
const ALBUM_ART_HEIGHT: i32 = 23; // Expected height of the album art image
const WIDGET_WIDTH: i32 = 250;
const MAX_TRACK_WIDTH: i32 = WIDGET_WIDTH - ALBUM_ART_WIDTH;

fn get_mpris_player_label_text() -> String {
    mpris::get_default_player()
        .map_or_else(|| "No MPRIS players".to_string(), |player| format!("{} - {}",
            player.metadata.artist.unwrap_or(vec!["No artist".to_string()]).join(", "),
            player.metadata.title.unwrap_or("No title".to_string())
        ))
}

pub fn new() -> gtk4::Box {
    let current_art_url = RefCell::new(" ".to_string());

    relm4_macros::view! {
        widget_middle_click_gesture = gesture::on_middle_click(|_, _, _| {
            if let Some(player) = mpris::get_default_player() {
                if let Err(e) = player.play_pause() {
                    eprintln!("Failed to toggle play/pause: {}", e);
                }
            } else {
                eprintln!("No MPRIS player available to toggle play/pause.");
            }
        }),

        widget_right_click_gesture = gesture::on_secondary_click(|_, _, _| {
            if let Some(player) = mpris::get_default_player() {
                if let Err(e) = player.next() {
                    eprintln!("Failed to skip to next track: {}", e);
                }
            } else {
                eprintln!("No MPRIS player available to skip to next track.");
            }
        }),

        widget_scroll_controller = gesture::on_vertical_scroll(|delta_y| {
            if let Some(player) = mpris::get_default_player() {
                let step = if delta_y < 0.0 {
                    VOLUME_STEP
                } else {
                    -VOLUME_STEP
                };
                
                player.adjust_volume(step).unwrap_or_else(|e| {
                    eprintln!("Failed to adjust volume: {}", e);
                });
            } else {
                eprintln!("No MPRIS player available to adjust volume.");
            }
        }),

        no_players_widget = gtk4::Label {
            set_css_classes: &["bar-mpris-track"],
            set_label: "No MPRIS players",
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
            set_height_request: ALBUM_ART_HEIGHT
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
            set_css_classes: &["bar-widget", "bar-mpris"],
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
                    blank_pixbuf.fill(0x0D0D0DFF);
                    current_album_art.set_from_pixbuf(Some(&blank_pixbuf));
                } else {
                    eprintln!("Failed to create blank pixbuf for album art! :O");
                }
            };

            if let Some(art_url) = default_player.metadata.art_url {
                if *current_art_url.borrow() != *art_url {
                    *current_art_url.borrow_mut() = art_url.to_string();

                    // URL-decode the album art URL
                    let art_url = match urlencoding::decode(&art_url.replace("file://", "")) {
                        Ok(decoded) => decoded.into_owned(),
                        Err(e) => {
                            eprintln!("Failed to decode album art URL: {}", e);
                            return;
                        }
                    };

                    // Make pixbuf from album art
                    let pixbuf = gtk4::gdk_pixbuf::Pixbuf::from_file(&*art_url);

                    if let Ok(pixbuf) = pixbuf {
                        let scaled_pixbuf = pixbuf.scale_simple(ALBUM_ART_WIDTH, ALBUM_ART_HEIGHT, gtk4::gdk_pixbuf::InterpType::Tiles);
                        if let Some(scaled_pixbuf) = scaled_pixbuf {
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
                    } else {
                        eprintln!("Failed to load album art from file: {}", art_url);
                        make_blank_art();
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

    BarModuleWrapper::new(widget)
        .add_controller(widget_middle_click_gesture)
        .add_controller(widget_right_click_gesture)
        .add_controller(widget_scroll_controller)
        .get_widget()
}