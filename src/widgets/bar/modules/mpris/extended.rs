use futures_signals::signal_vec::VecDiff;
use gtk4::prelude::*;
use relm4::RelmIterChildrenExt as _;

use crate::singletons::mpris::{self, MPRIS, mpris_player::PlaybackStatus, set_default_player};
use crate::utils::gesture;
use super::progress;

pub static SEEK_STEP_MICROSECONDS: i64 = 5_000_000;

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
    if let Some(default_player) = mpris::get_default_player()
        && let Some(art_url) = default_player.metadata.art_url
    {
        let css = format!(
            ".bar-mpris-extended-background {{ background-image: url('{}'); }}",
            art_url.replace('\'', "\\'")
        );

        Some(css)
    } else {
        None
    }
}

fn default_mpris_player() -> gtk4::Box {
    let background_style_provider = gtk4::CssProvider::new();
    let progress_bar = progress::ProgressBar::new();

    view! {
        previous_button = gtk4::Button {
            set_css_classes: &["bar-mpris-button"],
            set_label: "skip_previous",
            set_hexpand: false,
            connect_clicked => |_| {
                mpris::with_default_player_mut(|player| if let Err(e) = player.previous() {
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
            append: &progress_bar.drawing_area,
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

        let seek_amount = if delta < 0.0 {
            SEEK_STEP_MICROSECONDS
        } else {
            -SEEK_STEP_MICROSECONDS
        };

        if let Err(e) = player.seek(seek_amount) {
            eprintln!("Failed to seek: {}", e);
        }
    }));

    mpris::subscribe_to_default_player_changes({
        let progress = progress.clone();
        let progress_bar = progress_bar.clone();
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
                    progress_bar.set_position(position as f64);
                    progress_bar.set_duration(duration as f64);
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
                    let progress_bar = progress_bar.clone();
                    move || {
                        mpris::with_default_player_mut(|player| {
                            if player.playback_status == PlaybackStatus::Playing
                                && let Ok(position) = player.get_and_update_position()
                                && let Some(duration) = player.metadata.length
                            {
                                progress.set_label(&format!("{} / {}", format_duration(position), format_duration(duration)));
                                progress_bar.set_position(position as f64);
                                progress_bar.set_duration(duration as f64);
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

fn players_list_item(player: &mpris::mpris_player::MprisPlayer, index: usize) -> gtk4::Button {
    // <icon icon={player.get_bus_name().split(".")[player.get_bus_name().split(".").length - 1]}/>
    let identifier = player.bus.split('.').next_back().unwrap_or("emote-love");

    view! {
        child = gtk4::Button {
            set_css_classes: &["bar-mpris-players-list-item"],
            set_hexpand: false,
            connect_clicked => move |_| set_default_player(index),

            gtk4::Image {
                set_css_classes: &["bar-mpris-players-list-item-icon"],
                set_icon_name: Some(identifier),
                set_pixel_size: 16,
                set_halign: gtk4::Align::Center,
            },
        }
    }

    child
}

fn players_list() -> gtk4::Box {
    let bx = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    bx.set_css_classes(&["bar-mpris-players-list"]);
    bx.set_hexpand(true);

    mpris::subscribe_to_player_list_changes({
        let bx = bx.clone();
        move |difference, size| {
            match difference {
                VecDiff::Push { value } => {
                    let row = players_list_item(&value, size);
                    bx.append(&row);
                }
                
                VecDiff::RemoveAt { index, .. } => {
                    if let Some(child) = bx.iter_children().nth(index) {
                        bx.remove(&child);
                    }
                }

                VecDiff::Pop {} => {
                    if let Some(child) = bx.iter_children().last() {
                        bx.remove(&child);
                    }
                }

                VecDiff::Clear {} => {
                    bx.iter_children().for_each(|child| {
                        bx.remove(&child);
                    });
                }

                _ => {}
            }

            if size < 1 {
                bx.hide();
            } else {
                bx.show();
            }
        }
    });

    mpris::subscribe_to_default_player_changes({
        let bx = bx.clone();
        move |index| {
            bx.iter_children().for_each(|child| {
                child.set_css_classes(&["bar-mpris-players-list-item"]);
            });

            if let Some(selected_child) = bx.iter_children().nth(index) {
                selected_child.set_css_classes(&["bar-mpris-players-list-item", "is-default"]);
            }
        }
    });

    bx.add_controller(gesture::on_vertical_scroll(|delta_y| {
        let delta_index = if delta_y < 0.0 { 1 } else { -1 };
        let current_index = MPRIS.default_player.get() as isize;
        let players_count = MPRIS.players.lock_ref().len() as isize;

        if players_count > 0 {
            let new_index = (current_index + delta_index + players_count) % players_count;
            set_default_player(new_index as usize);
        }
    }));

    bx
}

pub fn extended() -> gtk4::Box {
    view! {
        widget = gtk4::Box {
            set_css_classes: &["bar-mpris-extended"],
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 0,

            append: &default_mpris_player(),
            append: &players_list(),
        },
    }

    widget
}