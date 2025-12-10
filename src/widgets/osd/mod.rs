use gdk4::cairo::Region;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use self::imp::Osd;

pub mod imp;

pub struct OsdWindow {
    pub window: gtk4::ApplicationWindow,
    pub container: gtk4::Box,
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
        }
    }

    pub fn add_osd(&self, osd: &impl Osd) {
        self.container.append(&osd.make_revealer().reveal);
    }
}