use freedesktop_desktop_entry::get_languages_from_env;
use gtk4::prelude::*;

use crate::{ipc, singletons::apps::{self, pixbuf::get_pixbuf_or_fallback}};

pub mod frequent;

pub fn make_item_from_command(command: &str) -> Option<gtk4::Button> {
    let locales = get_languages_from_env();
    let entry = apps::get_from_command(command)?;
    let icon_pixbuf = get_pixbuf_or_fallback(entry.icon().unwrap_or_default(), "emote-love");

    view! {
        button = gtk4::Button {
            set_css_classes: &["overview-window-button"],
            connect_clicked: {
                let command = command.to_owned();
                move |_| {
                    apps::launch_and_track(&command);

                    // Hide the overview after clicking an item
                    let _ = ipc::client::send_message("hide_overview");
                }
            },

            gtk4::Box {
                set_css_classes: &["overview-window-button-box"],
                set_orientation: gtk4::Orientation::Horizontal,
                set_hexpand: true,

                gtk4::Image {
                    set_from_pixbuf: icon_pixbuf.as_ref(),
                    set_pixel_size: 24,
                    set_css_classes: &["overview-window-button-icon"],
                },

                gtk4::Label {
                    set_label: entry.name(&locales).as_ref().map_or("Unnamed", |v| v),
                    set_css_classes: &["overview-window-button-label"],
                    set_halign: gtk4::Align::Start,
                    set_valign: gtk4::Align::Center,
                }
            }
        }
    }

    Some(button)
}