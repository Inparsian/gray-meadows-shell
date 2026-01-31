use std::{rc::Rc, cell::RefCell};
use gtk4::prelude::*;

use crate::services::wireplumber;
use super::{OsdRevealer, Osd};

#[derive(Debug, Clone)]
pub struct VolumeOsd {
    pub revealers: Rc<RefCell<Vec<OsdRevealer>>>,
}

impl Osd for VolumeOsd {
    fn key() -> &'static str {
        "Volume"
    }

    fn make_revealer(&self) -> OsdRevealer {
        let revealer = OsdRevealer::default();
        revealer.header_key.set_text(Self::key());
        revealer.levelbar.set_visible(true);
        revealer.levelbar.set_value(0.0);
        revealer.levelbar.set_min_value(0.0);
        revealer.levelbar.set_max_value(100.0);

        self.revealers.borrow_mut().push(revealer.clone());
        revealer
    }

    fn listen_for_events(&self) {
        wireplumber::subscribe_default_speaker_volume({
            let revealers = self.revealers.clone();
            move |volume: f32| {
                for revealer in &*revealers.borrow() {
                    revealer.header_value.set_text(&format!("{:.0}%", volume * 100.0));
                    revealer.levelbar.set_value((volume * 100.0).into());
                    revealer.reveal();
                }
            }
        });
    }
}

impl Default for VolumeOsd {
    fn default() -> Self {
        let osd = Self {
            revealers: Rc::new(RefCell::new(vec![])),
        };

        osd.listen_for_events();
        osd
    }
}