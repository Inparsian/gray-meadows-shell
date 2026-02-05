use gtk::prelude::*;
use cairo::Region;
use gtk4_layer_shell::{Edge, Layer, LayerShell as _};

use self::imp::Osd;

pub mod imp;

pub struct OsdWindow {
    pub window: gtk::ApplicationWindow,
    pub container: gtk::Box,
}

impl OsdWindow {
    pub fn new(application: &libadwaita::Application, monitor: &gdk::Monitor) -> Self {
        view! {
            container = gtk::Box {
                set_css_classes: &["osd-container"],
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 0,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::End,
            },

            window = gtk::ApplicationWindow {
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
            if let Some(surface) = win.native().and_then(|n| n.surface()) {
                surface.set_input_region(&Region::create());
            }
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