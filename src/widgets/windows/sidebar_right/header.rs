use std::path::Path;
use gtk4::prelude::*;
use futures_signals::signal::SignalExt as _;

use crate::USERNAME;
use crate::widgets::windows;
use crate::filesystem;
use crate::singletons::sysstats::SYS_STATS;

fn parse_uptime_seconds(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    format!(
        "{}{}{}{}s",
        if days > 0 { format!("{}d ", days) } else { String::new() },
        if hours > 0 { format!("{}h ", hours) } else { String::new() },
        if minutes > 0 { format!("{}m ", minutes) } else { String::new() },
        secs
    )
}

fn format_uptime_seconds(uptime: u64) -> String {
    format!("  up: {}", parse_uptime_seconds(uptime))
}

fn get_uptime_label_text(uptime: Option<u64>) -> String {
    uptime.map_or_else(
        || format_uptime_seconds(SYS_STATS.uptime.get()),
        format_uptime_seconds
    )
}

pub fn new() -> gtk4::Box {
    view! {
        face = gtk4::Image {
            set_pixel_size: 40,
            set_css_classes: &["sidebar-right-header-icon", "generic"],
        },

        uptime_label = gtk4::Label {
            set_label: &get_uptime_label_text(None),
            set_css_classes: &["sidebar-right-header-sublabel"],
            set_xalign: 0.0
        },

        header = gtk4::Box {
            set_css_classes: &["right-sidebar-header"],
            set_orientation: gtk4::Orientation::Horizontal,
            set_spacing: 12,
            set_hexpand: true,
            
            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_hexpand: true,
                set_halign: gtk4::Align::Start,
                append: &face,

                gtk4::Box {
                    set_orientation: gtk4::Orientation::Vertical,
                    set_valign: gtk4::Align::Center,
                    set_spacing: 2,

                    gtk4::Label {
                        set_label: &USERNAME, 
                        set_css_classes: &["sidebar-right-header-label"],
                        set_xalign: 0.0
                    },

                    append: &uptime_label
                }
            },

            gtk4::Button {
                set_css_classes: &["sidebar-button"],
                set_halign: gtk4::Align::End,
                set_valign: gtk4::Align::Center,
                connect_clicked: move |_| {
                    windows::hide("right_sidebar");
                    windows::toggle("session");
                },

                gtk4::Label {
                    set_css_classes: &["sidebar-button-label", "generic"],
                    set_label: "gtfo",
                    set_xalign: 0.5,
                    set_halign: gtk4::Align::Center
                }
            }
        }
    }

    // If /home/USER/.face exists, use it as the profile picture
    let face_path = format!("{}/.face", filesystem::get_home_directory());
    if Path::new(&face_path).exists() {
        face.set_from_file(Some(face_path));
    }

    gtk4::glib::spawn_future_local(signal!(SYS_STATS.uptime, (uptime) {
        uptime_label.set_label(&get_uptime_label_text(Some(uptime)));
    }));

    header
}