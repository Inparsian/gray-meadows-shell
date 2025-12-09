use std::collections::HashMap;
use gdk4::cairo::Region;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use crate::{APP_LOCAL, widgets::osd::imp::Osd};

pub mod imp;

#[derive(Debug, Clone)]
pub struct OsdWindow {
    pub window: gtk4::ApplicationWindow,
    pub container: gtk4::Box,
    pub osds: HashMap<String, imp::Osd>,
}

impl OsdWindow {
    pub fn new(application: &libadwaita::Application, monitor: &gdk4::Monitor) -> Self {
        view! {
            container = gtk4::Box {
                set_css_classes: &["osd-container"],
                set_orientation: gtk4::Orientation::Vertical,
                set_spacing: 0,
                set_halign: gtk4::Align::Center,
                set_valign: gtk4::Align::End,
            },

            window = gtk4::ApplicationWindow {
                set_css_classes: &["osd-window"],
                set_application: Some(application),
                init_layer_shell: (),
                set_monitor: Some(monitor),
                set_layer: Layer::Overlay,
                set_anchor: (Edge::Bottom, true),
                set_namespace: Some("gms-osd"),
                set_child: Some(&container),
            }
        }

        window.connect_visible_notify(move |win| {
            let Some(native) = win.native() else {
                return;
            };

            let Some(surface) = native.surface() else {
                return;
            };

            surface.set_input_region(&Region::create());
        });

        OsdWindow {
            window,
            container,
            osds: HashMap::new(),
        }
    }

    pub fn add_osd(&mut self, key: &str, osd: imp::Osd) {
        self.container.append(&osd.reveal);
        self.osds.insert(key.to_owned(), osd);
    }

    pub fn get_osd(&self, key: &str) -> Option<&imp::Osd> {
        self.osds.get(key)
    }
}

pub fn get_osd(key: &str) -> Vec<Osd> {
    APP_LOCAL.with(|app| {
        let containers = &app.borrow().osd_containers;
        let mut result = Vec::new();

        for osd_container in containers.borrow().iter() {
            if let Some(osd) = osd_container.get_osd(key) {
                result.push(osd.clone());
            }
        }

        result
    })
}