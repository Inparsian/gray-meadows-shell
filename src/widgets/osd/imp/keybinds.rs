use futures_signals::signal::SignalExt;

use crate::{singletons::hyprland, timeout::Timeout, widgets::osd::imp::{Osd, OsdRevealer}};

#[derive(Debug, Clone)]
pub struct KeybindsOsd {
    pub revealer: OsdRevealer,
    pub timeout: Timeout
}

impl super::Osd for KeybindsOsd {
    fn key() -> &'static str {
        "Keybinds"
    }

    fn revealer(&self) -> &OsdRevealer {
        &self.revealer
    }

    fn listen_for_events(&self) {
        gtk4::glib::spawn_future_local({
            let value_label = self.revealer().header_value.clone();
            let revealer = self.revealer.clone();

            signal_cloned!(hyprland::HYPRLAND.submap, (submap) {
                if let Some(submap) = submap {
                    if submap == "grab" {
                        value_label.set_text("off");
                    } else {
                        value_label.set_text("on");
                    }
                } else {
                    // Assume that the absence of a submap means that the grab submap is not active
                    value_label.set_text("on");
                }

                revealer.reveal();
            })
        });
    }
}

impl Default for KeybindsOsd {
    fn default() -> Self {
        let revealer = OsdRevealer::default();
        revealer.header_key.set_text(Self::key());

        Self {
            revealer,
            timeout: Timeout::default()
        }
    }
}