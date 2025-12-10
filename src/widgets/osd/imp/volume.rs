use std::{rc::Rc, cell::RefCell};
use gtk4::prelude::*;

use crate::{ffi::astalwp::WpEvent, singletons::wireplumber, widgets::osd::imp::{Osd, OsdRevealer}};

#[derive(Debug, Clone)]
pub struct VolumeOsd {
    pub revealers: Rc<RefCell<Vec<OsdRevealer>>>,
}

impl super::Osd for VolumeOsd {
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
        // We can't initialize the volume immediately because the WirePlumber singleton
        // might not be ready yet. Subscribe to events.
        gtk4::glib::spawn_future_local({
            let revealers = self.revealers.clone();
            
            async move {
                let set = |volume: f32| {
                    for revealer in &*revealers.borrow() {
                        revealer.header_value.set_text(&format!("{:.0}%", volume * 100.0));
                        revealer.levelbar.set_value((volume * 100.0).into());
                        revealer.reveal();
                    }
                };

                while let Ok(event) = wireplumber::subscribe().recv().await {
                    match event {
                        WpEvent::CreateSpeaker(endpoint) => {
                            if endpoint.is_default {
                                set(endpoint.node.volume);
                            }
                        },
                    
                        WpEvent::RemoveSpeaker(endpoint) => {
                            if let Some(default_speaker) = wireplumber::get_default_speaker() {
                                if default_speaker.node.id == endpoint.node.id {
                                    set(default_speaker.node.volume);
                                }
                            }
                        },
                    
                        WpEvent::UpdateDefaultSpeaker(id) => {
                            if let Some(speaker) = wireplumber::get_endpoint(id) {
                                set(speaker.node.volume);
                            }
                        },
                    
                        WpEvent::UpdateEndpoint(id, property_name) => if property_name == "volume" {
                            if let Some(default_speaker) = wireplumber::get_default_speaker() {
                                if default_speaker.node.id == id {
                                    set(default_speaker.node.volume);
                                }
                            }
                        },
                    
                        _ => {}
                    }
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