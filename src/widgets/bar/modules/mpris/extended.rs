use gtk4::prelude::*;

use crate::{helpers::gesture, singletons::mpris::{self, mpris_player::PlaybackStatus}};

fn format_artist_list(artists: &[String]) -> String {
    artists.join(", ")
}

fn format_duration(microseconds: i64) -> String {
    let total_seconds = microseconds / 1_000_000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{}:{:02}", minutes, seconds)
}

fn get_background_css() -> Option<String> {
    if let Some(default_player) = mpris::get_default_player() {
        if let Some(art_url) = default_player.metadata.art_url {
            let css = format!(
                ".bar-mpris-extended-background {{ background-image: url('{}'); }}",
                art_url.replace('\'', "\\'")
            );
            return Some(css);
        }
    }
    None
}

fn default_mpris_player() -> gtk4::Box {
    let background_style_provider = gtk4::CssProvider::new();

    view! {
        previous_button = gtk4::Button {
            set_css_classes: &["bar-mpris-button"],
            set_label: "skip_previous",
            set_hexpand: false,
            connect_clicked => |_| {
                mpris::with_default_player_mut(|player|  if let Err(e) = player.previous() {
                    eprintln!("Failed to skip to previous track: {}", e);
                });
            }
        },

        play_pause_button = gtk4::Button {
            set_css_classes: &["bar-mpris-button"],
            set_label: if mpris::get_default_player().is_some_and(|p| p.playback_status == PlaybackStatus::Playing) {
                "pause"
            } else {
                "play_arrow"
            },
            set_hexpand: false,
            connect_clicked => |_| {
                let Some(player) = mpris::get_default_player() else {
                    return eprintln!("No MPRIS player available to toggle play/pause.");
                };

                if let Err(e) = player.play_pause() {
                    eprintln!("Failed to toggle play/pause: {}", e);
                }
            }
        },

        next_button = gtk4::Button {
            set_css_classes: &["bar-mpris-button"],
            set_label: "skip_next",
            set_hexpand: false,
            connect_clicked => |_| {
                mpris::with_default_player_mut(|player| if let Err(e) = player.next() {
                    eprintln!("Failed to skip to next track: {}", e);
                });
            }
        },

        loop_button = gtk4::Button {
            set_css_classes: &["bar-mpris-button"],
            set_label: "repeat",
            set_hexpand: false,
            connect_clicked => |_| {
                let Some(player) = mpris::get_default_player() else {
                    return eprintln!("No MPRIS player available to change loop status.");
                };

                let new_status = match player.loop_status {
                    mpris::mpris_player::LoopStatus::None => mpris::mpris_player::LoopStatus::Playlist,
                    mpris::mpris_player::LoopStatus::Playlist => mpris::mpris_player::LoopStatus::Track,
                    mpris::mpris_player::LoopStatus::Track => mpris::mpris_player::LoopStatus::None,
                };
                
                if let Err(e) = player.set_loop_status(new_status) {
                    eprintln!("Failed to set loop status: {}", e);
                }
            }
        },

        shuffle_button = gtk4::Button {
            set_css_classes: &["bar-mpris-button"],
            set_label: "shuffle",
            set_hexpand: false,
            connect_clicked => |_| {
                let Some(player) = mpris::get_default_player() else {
                    return eprintln!("No MPRIS player available to toggle shuffle.");
                };

                let new_shuffle = !player.shuffle;
                
                if let Err(e) = player.set_shuffle(new_shuffle) {
                    eprintln!("Failed to set shuffle: {}", e);
                }
            }
        },

        background = gtk4::Box {
            set_css_classes: &["bar-mpris-extended-background"],
            set_hexpand: true,
            set_vexpand: true
        },

        buttons = gtk4::Box {
            set_orientation: gtk4::Orientation::Horizontal,
            set_spacing: 4,
            set_halign: gtk4::Align::Start,
            set_valign: gtk4::Align::End,

            append: &previous_button,
            append: &play_pause_button,
            append: &next_button,
            append: &loop_button,
            append: &shuffle_button,
        },

        progress = gtk4::Label {
            set_css_classes: &["bar-mpris-extended-progress"],
            set_hexpand: true,
            set_xalign: 1.0,
            set_label: "0:00 / 0:00",
        },

        controls = gtk4::CenterBox {
            set_css_classes: &["bar-mpris-extended-controls"],
            set_orientation: gtk4::Orientation::Horizontal,
            set_hexpand: true,

            set_start_widget: Some(&buttons),
            set_end_widget: Some(&progress),
        },

        metadata_artist = gtk4::Label {
            set_css_classes: &["bar-mpris-extended-artist"],
            set_hexpand: true,
            set_xalign: 0.0,
            set_ellipsize: gtk4::pango::EllipsizeMode::End,
        },

        metadata_title = gtk4::Label {
            set_css_classes: &["bar-mpris-extended-title"],
            set_hexpand: true,
            set_xalign: 0.0,
            set_ellipsize: gtk4::pango::EllipsizeMode::End,
        },

        metadata_album = gtk4::Label {
            set_css_classes: &["bar-mpris-extended-album"],
            set_hexpand: true,
            set_xalign: 0.0,
            set_ellipsize: gtk4::pango::EllipsizeMode::End,
        },

        metadata = gtk4::Box {
            set_css_classes: &["bar-mpris-extended-metadata"],
            set_orientation: gtk4::Orientation::Vertical,
            set_hexpand: true,
            set_vexpand: true,

            append: &metadata_artist,
            append: &metadata_title,
            append: &metadata_album,
        },

        over = gtk4::Box {
            set_css_classes: &["bar-mpris-extended-box"],
            set_orientation: gtk4::Orientation::Vertical,
            set_hexpand: true,

            append: &metadata,
            append: &controls,
        },

        player_overlay = gtk4::Overlay {
            set_child: Some(&background),
            set_overflow: gtk4::Overflow::Hidden,
            add_overlay: &over,
        },

        no_players_widget = gtk4::Label {
            set_css_classes: &["bar-mpris-extended-no-players"],
            set_label: "No players",
            set_hexpand: true,
            set_xalign: 0.5
        },

        widget = gtk4::Box {
            set_css_classes: &["bar-mpris-extended-container"],
            set_hexpand: true,

            append: &no_players_widget,
            append: &player_overlay,
        },
    }

    background.style_context().add_provider(&background_style_provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

    progress.add_controller(gesture::on_vertical_scroll(|delta| {
        let Some(player) = mpris::get_default_player() else {
            return eprintln!("No MPRIS player available to seek.");
        };

        let step_microseconds = 5_000_000; // 5 seconds in microseconds
        let seek_amount = if delta < 0.0 {
            step_microseconds
        } else {
            -step_microseconds
        };

        if let Err(e) = player.seek(seek_amount) {
            eprintln!("Failed to seek: {}", e);
        }
    }));

    mpris::subscribe_to_default_player_changes({
        let progress = progress.clone();
        move |_| {
            if let Some(default_player) = mpris::get_default_player() {
                no_players_widget.hide();
                player_overlay.show();

                let is_playing = default_player.playback_status == PlaybackStatus::Playing;
                play_pause_button.set_label(if is_playing { "pause" } else { "play_arrow" });

                if default_player.shuffle {
                    shuffle_button.set_css_classes(&["bar-mpris-button", "toggled"]);
                } else {
                    shuffle_button.set_css_classes(&["bar-mpris-button"]);
                }

                loop_button.set_label(if default_player.loop_status == mpris::mpris_player::LoopStatus::Track {"repeat_one"} else {"repeat"});
                if default_player.loop_status != mpris::mpris_player::LoopStatus::None {
                    loop_button.set_css_classes(&["bar-mpris-button", "toggled"]);
                } else {
                    loop_button.set_css_classes(&["bar-mpris-button"]);
                }

                if let (position, Some(duration)) = (default_player.position, default_player.metadata.length) {
                    progress.set_label(&format!("{} / {}", format_duration(position), format_duration(duration)));
                }

                // update metadata labels
                metadata_title.set_label(default_player.metadata.title.as_deref().unwrap_or("Unknown Title"));
                metadata_artist.set_label(&format_artist_list(&default_player.metadata.artist.unwrap_or_default()));
                metadata_album.set_label(default_player.metadata.album.as_deref().unwrap_or("Unknown Album"));

                // set the background of background box to the album art if any
                if let Some(css) = get_background_css() {
                    background_style_provider.load_from_data(&css);
                } else {
                    background_style_provider.load_from_data("");
                }
            } else {
                no_players_widget.show();
                player_overlay.hide();
                background_style_provider.load_from_data("");
            }
        }
    });

    // run a future that changes the progress bar value every second if playing
    gtk4::glib::spawn_future_local({
        async move {
            loop {
                gtk4::glib::source::idle_add_local_once({
                    let progress = progress.clone();
                    move || {
                        mpris::with_default_player_mut(|player| {
                            if player.playback_status == PlaybackStatus::Playing {
                                if let (Ok(position), Some(duration)) = (player.get_and_update_position(), player.metadata.length) {
                                    progress.set_label(&format!("{} / {}", format_duration(position), format_duration(duration)));
                                }
                            }
                        });
                    }
                });

                gtk4::glib::timeout_future(std::time::Duration::from_millis(500)).await;
            }
        }
    });

    widget
}

pub fn extended() -> gtk4::Box {
    view! {
        widget = gtk4::Box {
            set_css_classes: &["bar-mpris-extended"],
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 4,

            append: &default_mpris_player(),
        },
    }

    widget
}