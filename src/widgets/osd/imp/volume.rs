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
        let (tx, rx) = async_channel::unbounded::<f32>();
        tokio::spawn(async move {
            while let Ok(event) = wireplumber::subscribe().recv().await {
                match event {
                    WpEvent::CreateSpeaker(endpoint) => {
                        if endpoint.is_default {
                            let _ = tx.send(endpoint.node.volume).await;
                        }
                    },
                
                    WpEvent::RemoveSpeaker(endpoint) => {
                        if let Some(default_speaker) = wireplumber::get_default_speaker() {
                            if default_speaker.node.id == endpoint.node.id {
                                let _ = tx.send(default_speaker.node.volume).await;
                            }
                        }
                    },
                
                    WpEvent::UpdateDefaultSpeaker(id) => {
                        if let Some(speaker) = wireplumber::get_endpoint(id) {
                            let _ = tx.send(speaker.node.volume).await;
                        }
                    },
                
                    WpEvent::UpdateEndpoint(id, property_name) => if property_name == "volume" {
                        if let Some(default_speaker) = wireplumber::get_default_speaker() {
                            if default_speaker.node.id == id {
                                let _ = tx.send(default_speaker.node.volume).await;
                            }
                        }
                    },
                
                    _ => {}
                }
            }
        });

        gtk4::glib::spawn_future_local({
            let revealers = self.revealers.clone();

            async move {
                while let Ok(volume) = rx.recv().await {
                    for revealer in &*revealers.borrow() {
                        revealer.header_value.set_text(&format!("{:.0}%", volume * 100.0));
                        revealer.levelbar.set_value((volume * 100.0).into());
                        revealer.reveal();
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