use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use crate::{reactivity, singletons};

pub struct Bar {
    pub window: gtk4::ApplicationWindow,
}

impl Bar {
    pub fn new(application: &gtk4::Application, monitor: &gdk4::Monitor) -> Self {
        relm4_macros::view! {
            window = gtk4::ApplicationWindow {
                set_application: Some(application),
                init_layer_shell: (),
                set_monitor: Some(&monitor),
                set_default_height: 33,
                set_layer: Layer::Top,
                set_anchor: (Edge::Left, true),
                set_anchor: (Edge::Right, true),
                set_anchor: (Edge::Top, true),
                auto_exclusive_zone_enable: (),

                gtk4::Box {
                    set_css_classes: &["bar"],
                    set_spacing: 1,

                    gtk4::Box {
                        set_css_classes: &["bar-widget"],
                        set_halign: gtk4::Align::Start,

                        gtk4::Label {
                            set_label: "Gray Meadows Shell",
                            set_hexpand: true,
                            set_xalign: 0.5
                        }
                    },

                    gtk4::Box {
                        set_css_classes: &["bar-widget"],
                        set_halign: gtk4::Align::End,

                        reactivity::reactive_label(singletons::date_time::DATE_TIME.time.clone()) {
                            set_hexpand: true,
                            set_xalign: 0.5
                        }
                    }
                }
            }
        }

        Bar {
            window
        }
    }
}