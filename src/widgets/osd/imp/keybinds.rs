use std::{rc::Rc, cell::RefCell};
use futures_signals::signal::SignalExt;

use crate::singletons::hyprland;
use super::{Osd, OsdRevealer};

#[derive(Debug, Clone)]
pub struct KeybindsOsd {
    pub revealers: Rc<RefCell<Vec<OsdRevealer>>>,
}

impl super::Osd for KeybindsOsd {
    fn key() -> &'static str {
        "Keybinds"
    }

    fn make_revealer(&self) -> OsdRevealer { 
        let revealer = OsdRevealer::default();
        revealer.header_key.set_text(Self::key());
        
        self.revealers.borrow_mut().push(revealer.clone());
        revealer
    }

    fn listen_for_events(&self) {
        gtk4::glib::spawn_future_local({
            let revealers = self.revealers.clone();

            signal_cloned!(hyprland::HYPRLAND.submap, (submap) {
                let value = submap.map_or("on", |submap| if submap == "grab" {
                    "off"
                } else {
                    "on"
                });

                for revealer in &*revealers.borrow() {
                    revealer.header_value.set_text(value);
                    revealer.reveal();
                }
            })
        });
    }
}

impl Default for KeybindsOsd {
    fn default() -> Self {
        let osd = Self {
            revealers: Rc::new(RefCell::new(vec![])),
        };

        osd.listen_for_events();
        osd
    }
}