pub mod volume;
pub mod keybinds;

use std::time::Duration;
use gtk4::prelude::*;

use crate::utils::timeout::Timeout;
use crate::widgets::common::revealer::{AdwRevealer, AdwRevealerDirection, GEasing};

static TRANSITION_DURATION_MS: u32 = 600;
static DISPLAY_DURATION: f64 = 2.0;

pub trait Osd {
    fn key() -> &'static str;
    fn make_revealer(&self) -> OsdRevealer;
    fn listen_for_events(&self);
}

#[derive(Debug, Clone)]
pub struct OsdRevealer {
    timeout: Timeout,
    pub header_key: gtk4::Label,
    pub header_value: gtk4::Label,
    pub levelbar: gtk4::LevelBar,
    pub reveal: AdwRevealer,
}

impl Default for OsdRevealer {
    fn default() -> Self {
        view! {
            header_key = gtk4::Label {
                set_css_classes: &["osd-header-key"],
            },

            header_value = gtk4::Label {
                set_css_classes: &["osd-header-value"],
            },

            header_centerbox = gtk4::CenterBox {
                set_orientation: gtk4::Orientation::Horizontal,
                set_start_widget: Some(&header_key),
                set_center_widget: Some(&gtk4::Box::new(gtk4::Orientation::Horizontal, 0)),
                set_end_widget: Some(&header_value),
            },

            levelbar = gtk4::LevelBar {
                set_css_classes: &["osd-levelbar"],
                set_min_value: 0.0,
                set_max_value: 1.0,
                set_height_request: 14,
                set_value: 0.0,
                set_visible: false,
            },

            bx = gtk4::Box {
                set_css_classes: &["osd-box"],
                set_orientation: gtk4::Orientation::Vertical,
                set_spacing: 0,

                append: &header_centerbox,
                append: &levelbar,
            },

            reveal = AdwRevealer {
                set_css_classes: &["osd-item"],
                set_reveal: false,
                set_transition_direction: AdwRevealerDirection::Up,
                set_transition_duration: TRANSITION_DURATION_MS,
                set_show_easing: GEasing::EaseOutExpo,
                set_hide_easing: GEasing::EaseInOutBack,
                set_halign: gtk4::Align::Center,
                set_valign: gtk4::Align::End,
                set_child_from: Some(&bx),
            }
        }

        Self {
            timeout: Timeout::default(),
            header_key,
            header_value,
            levelbar,
            reveal,
        }
    }
}

impl OsdRevealer {
    pub fn reveal(&self) {
        self.reveal.add_css_class("revealed");
        self.reveal.set_reveal(true);

        self.timeout.set(Duration::from_secs_f64(DISPLAY_DURATION), {
            let reveal = self.reveal.clone();
            move || {
                reveal.remove_css_class("revealed");
                reveal.set_reveal(false);
            }
        });
    }
}