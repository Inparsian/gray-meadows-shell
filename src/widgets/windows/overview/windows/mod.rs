use freedesktop_desktop_entry::get_languages_from_env;
use gtk::prelude::*;

use crate::pixbuf;
use crate::widgets::windows;
use crate::services::apps;

pub mod frequent;
pub mod recent;

pub fn build_window(label: &str) -> (gtk::Box, gtk::Box) {
    let widget = gtk::Box::new(gtk::Orientation::Vertical, 12);
    widget.set_css_classes(&["overview-window"]);
    widget.set_hexpand(true);
    widget.set_vexpand(true);
    widget.set_halign(gtk::Align::Center);
    widget.set_valign(gtk::Align::Fill);

    let header = gtk::Label::new(Some(label));
    header.set_css_classes(&["overview-window-header"]);
    widget.append(&header);

    let children = gtk::Box::new(gtk::Orientation::Vertical, 0);
    children.set_css_classes(&["overview-window-children"]);
    children.set_hexpand(true);
    children.set_vexpand(true);
    widget.append(&children);

    (widget, children)
}

pub fn make_item_from_command(command: &str) -> Option<gtk::Button> {
    let locales = get_languages_from_env();
    let entry = apps::get_from_command(command)?;
    let icon_pixbuf = pixbuf::get_pixbuf_or_fallback(entry.icon().unwrap_or_default(), "emote-love");

    view! {
        button = gtk::Button {
            set_css_classes: &["overview-window-button"],
            connect_clicked: {
                let command = command.to_owned();
                move |_| {
                    apps::launch_and_track(&command);

                    // Hide the overview after clicking an item
                    windows::hide("overview");
                }
            },

            gtk::Box {
                set_css_classes: &["overview-window-button-box"],
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,

                gtk::Image {
                    set_from_pixbuf: icon_pixbuf.as_ref(),
                    set_pixel_size: 24,
                    set_css_classes: &["overview-window-button-icon"],
                },

                gtk::Label {
                    set_label: entry.name(&locales).as_ref().map_or("Unnamed", |v| v),
                    set_css_classes: &["overview-window-button-label"],
                    set_halign: gtk::Align::Start,
                    set_valign: gtk::Align::Center,
                }
            }
        }
    }

    Some(button)
}