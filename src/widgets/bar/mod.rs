pub mod wrapper;
pub mod module;
mod modules {
    pub mod workspaces;
    pub mod client;
    pub mod sysstats;
    pub mod mpris;
    pub mod clock;
    pub mod tray;
    pub mod volume;
}

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use crate::{APP_LOCAL, helpers::gesture, widgets::bar::module::BarModule};

static BAR_HEIGHT: i32 = 33;

pub struct BarWindow {
    pub window: gtk4::ApplicationWindow,
    pub modules: Vec<BarModule>,
}

impl BarWindow {
    pub fn new(application: &libadwaita::Application, monitor: &gdk4::Monitor) -> Self {
        view! {
            test_minimal = gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 0,
                set_halign: gtk4::Align::Center,
                set_valign: gtk4::Align::Center,

                gtk4::Label {
                    set_label: "Minimal"
                }
            },

            test_expanded = gtk4::Box {
                set_orientation: gtk4::Orientation::Vertical,
                set_spacing: 0,
                set_halign: gtk4::Align::Center,
                set_valign: gtk4::Align::Center,

                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
                gtk4::Label { set_label: "Expanded!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!" },
            },

            test_bar_module = gtk4::Box {
                set_css_classes: &["bar-widget"],
                set_orientation: gtk4::Orientation::Vertical,
                set_spacing: 1,

                append: &test_minimal,
                append: &test_expanded
            },

            test_bar_module_wrapper = gtk4::Box {
                set_css_classes: &["bar-widget-wrapper"],
                set_orientation: gtk4::Orientation::Horizontal,
                set_hexpand: false,
                set_valign: gtk4::Align::Start,

                append: &test_bar_module
            },

            left_box = gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 1,
                set_valign: gtk4::Align::Start,

                append: &modules::workspaces::new(),
                append: &modules::client::new()
            },

            center_box = gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 1,
                set_valign: gtk4::Align::Start,

                append: &modules::sysstats::new(),
                append: &test_bar_module_wrapper,
                append: &modules::mpris::new(),
                append: &modules::clock::new()
            },

            right_box = gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 1,
                set_valign: gtk4::Align::Start,

                append: &modules::tray::new(),
                append: &modules::volume::new(),
            },

            window = gtk4::ApplicationWindow {
                set_css_classes: &["bar-window"],
                set_application: Some(application),
                init_layer_shell: (),
                set_monitor: Some(monitor),
                set_default_height: BAR_HEIGHT,
                set_layer: Layer::Top,
                set_anchor: (Edge::Left, true),
                set_anchor: (Edge::Right, true),
                set_anchor: (Edge::Top, true),
                set_exclusive_zone: BAR_HEIGHT,
                set_namespace: Some("gms-bar"),

                gtk4::CenterBox {
                    set_css_classes: &["bar"],

                    // Left side widgets
                    set_start_widget: Some(&left_box),

                    // Center widgets
                    set_center_widget: Some(&center_box),

                    // Right side widgets
                    set_end_widget: Some(&right_box),
                }
            }
        }

        let test_module = BarModule::new(test_minimal.upcast(), test_expanded.upcast());
        test_bar_module_wrapper.add_controller(gesture::on_primary_full_press({
            let module = test_module.clone();
            move |_, _, _| {
                module.toggle_expanded();
            }
        }));

        let modules = vec![
            test_module,
        ];

        // collapse expanded modules when clicking outside of them
        window.add_controller(gesture::on_primary_up({
            let modules = modules.clone();
            move |_, x, y| {
                if y <= BAR_HEIGHT as f64 {
                    return;
                }

                let mut inside_any = false;
                for module in &modules {
                    if module.is_expanded() {
                        let allocation = module.expanded.allocation();
                        let x_in = x >= allocation.x() as f64 && x <= (allocation.x() + allocation.width()) as f64;
                        let y_in = y >= allocation.y() as f64 && y <= (allocation.y() + allocation.height()) as f64;
                        if x_in && y_in {
                            inside_any = true;
                            break;
                        }
                    }
                }

                if !inside_any {
                    APP_LOCAL.with(|app| {
                        for bar in app.borrow().bars.borrow().iter() {
                            bar.hide_all_expanded_modules();
                        }
                    });
                }
            }
        }));

        BarWindow {
            window,
            modules,
        }
    }

    pub fn hide_all_expanded_modules(&self) {
        for module in &self.modules {
            module.set_expanded(false);
        }
    }
}