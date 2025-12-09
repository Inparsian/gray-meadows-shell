use futures_signals::signal::SignalExt;

use crate::{singletons::hyprland, widgets::osd::imp::{Osd, OsdRevealer}};

#[derive(Debug, Clone)]
pub struct KeybindsOsd {
    pub revealer: OsdRevealer
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
            let revealer = self.revealer.clone();

            signal_cloned!(hyprland::HYPRLAND.submap, (submap) {
                if let Some(submap) = submap {
                    if submap == "grab" {
                        revealer.header_value.set_text("off");
                    } else {
                        revealer.header_value.set_text("on");
                    }
                } else {
                    // Assume that the absence of a submap means that the grab submap is not active
                    revealer.header_value.set_text("on");
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

        let osd = Self {
            revealer
        };

        osd.listen_for_events();
        osd
    }
}