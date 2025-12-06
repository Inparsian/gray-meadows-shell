use std::time::Duration;
use gtk4::prelude::*;

use crate::timeout::Timeout;

static TRANSITION_DURATION_MS: u32 = 200;
static DISPLAY_DURATION: f64 = 2.0;

#[derive(Debug, Clone)]
pub struct Osd {
    timeout: Timeout,
    pub reveal: gtk4::Revealer,
    pub key_label: gtk4::Label,
    pub value_label: gtk4::Label,
    pub levelbar: gtk4::LevelBar
}

impl Default for Osd {
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

            reveal = gtk4::Revealer {
                set_css_classes: &["osd-item"],
                set_reveal_child: false,
                set_transition_type: gtk4::RevealerTransitionType::SlideUp,
                set_transition_duration: TRANSITION_DURATION_MS,
                set_halign: gtk4::Align::Center,
                set_valign: gtk4::Align::End,
                set_child: Some(&bx),
            }
        }

        Osd {
            timeout: Timeout::default(),
            reveal,
            key_label: header_key,
            value_label: header_value,
            levelbar,
        }
    }
}

impl Osd {
    pub fn set_key(&self, key: &str) {
        self.key_label.set_label(key);
    }

    pub fn set_value(&self, value: &str) {
        self.value_label.set_label(value);
    }

    pub fn set_levelbar_range(&self, min: f64, max: f64) {
        self.levelbar.set_min_value(min);
        self.levelbar.set_max_value(max);
    }

    pub fn set_levelbar_value(&self, value: f64) {
        self.levelbar.set_value(value);
    }

    pub fn set_levelbar_visible(&self, visible: bool) {
        self.levelbar.set_visible(visible);
    }

    pub fn reveal(&self) {
        self.reveal.add_css_class("revealed");
        self.reveal.set_reveal_child(true);

        self.timeout.set(Duration::from_secs_f64(DISPLAY_DURATION), {
            let reveal = self.reveal.clone();
            move || {
                reveal.remove_css_class("revealed");
                reveal.set_reveal_child(false);
            }
        });
    }
}