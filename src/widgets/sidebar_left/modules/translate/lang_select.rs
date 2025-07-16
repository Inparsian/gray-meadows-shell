use gtk4::prelude::*;

use crate::widgets::sidebar_left::modules::translate::{send_ui_event, LanguageSelectReveal, UiEvent};

fn get_page_boxes() -> gtk4::Box {
    relm4_macros::view! {
        page_boxes = gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 12,
            set_hexpand: true,
            set_vexpand: true
        }
    }
    
    page_boxes
}

pub fn new(
    reveal_type: LanguageSelectReveal,
    sender: async_channel::Sender<UiEvent>,
) -> gtk4::Box {
    let _page_boxes = get_page_boxes();

    relm4_macros::view! {
        widget = gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 12,
            set_hexpand: true,

            gtk4::Label {
                set_label: if reveal_type == LanguageSelectReveal::Source {
                    "Select Source Language"
                } else {
                    "Select Target Language"
                },
            },

            gtk4::Button {
                set_label: "Close",
                set_css_classes: &["google-translate-button"],
                connect_clicked: {
                    let tx = sender.clone();
                    move |_| send_ui_event(UiEvent::LanguageSelectRevealChanged(LanguageSelectReveal::None), &tx)
                }
            }
        }
    };

    widget
}