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

use std::collections::HashMap;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell as _};

use crate::APP_LOCAL;
use crate::gesture;
use crate::ipc;
use crate::singletons::hyprland;
use self::module::{BarModuleWrapper, hide_all_expanded_modules};

static BAR_HEIGHT: i32 = 33;

pub struct BarWindow {
    pub window: gtk4::ApplicationWindow,
    pub monitor: gdk4::Monitor,
    pub steal_window: gtk4::ApplicationWindow,
    pub modules: HashMap<String, BarModuleWrapper>,
}

impl BarWindow {
    pub fn new(application: &libadwaita::Application, monitor: &gdk4::Monitor) -> Self {
        let mpris_module = modules::mpris::new();
        let sysstats_module = modules::sysstats::new();
        let mut modules = HashMap::new();
        modules.insert("mpris".to_owned(), mpris_module.clone());
        modules.insert("sysstats".to_owned(), sysstats_module.clone());

        view! {
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

                append: &sysstats_module.bx,
                append: &mpris_module.bx,
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
            },

            steal_window = gtk4::ApplicationWindow {
                set_visible: false,
                set_css_classes: &["bar-steal-window"],
                set_application: Some(application),
                init_layer_shell: (),
                set_monitor: Some(monitor),
                set_layer: Layer::Overlay,
                set_anchor: (Edge::Left, true),
                set_anchor: (Edge::Right, true),
                set_anchor: (Edge::Top, true),
                set_anchor: (Edge::Bottom, true),
                set_namespace: Some("gms-bar-steal")
            }
        }

        // collapse expanded modules when clicking outside of them
        window.add_controller(gesture::on_primary_full_press({
            let window = window.clone();
            let steal_window = steal_window.clone();
            let modules = modules.clone();
            move |_, (px, py), (rx, ry)| {
                if py > BAR_HEIGHT as f64 && ry > BAR_HEIGHT as f64 {
                    let mut inside_any = false;
                    for wrapper in modules.values() {
                        if wrapper.module.is_expanded() {
                            let mod_allocation = wrapper.bx.allocation();
                            let parent_allocation = wrapper.bx.parent().unwrap().allocation();
                            let allocation = gdk4::Rectangle::new(
                                mod_allocation.x() + parent_allocation.x(),
                                mod_allocation.y() + parent_allocation.y(),
                                mod_allocation.width(),
                                mod_allocation.height(),
                            );

                            let px_in = px >= allocation.x() as f64 && px <= (allocation.x() + allocation.width()) as f64;
                            let py_in = py >= allocation.y() as f64 && py <= (allocation.y() + allocation.height()) as f64;
                            let rx_in = rx >= allocation.x() as f64 && rx <= (allocation.x() + allocation.width()) as f64;
                            let ry_in = ry >= allocation.y() as f64 && ry <= (allocation.y() + allocation.height()) as f64;
                            if (px_in && py_in) || (rx_in && ry_in) {
                                inside_any = true;
                                break;
                            }
                        }
                    }

                    if !inside_any {
                        hide_all_expanded_modules();
                    }
                }

                // if any are expanded at this point, activate the steal window
                let any_expanded = modules.values().any(|wrapper| wrapper.module.is_expanded());
                if any_expanded {
                    steal_window.set_visible(true);
                    window.set_layer(Layer::Overlay);
                    window.set_keyboard_mode(KeyboardMode::OnDemand);
                }
            }
        }));

        window.add_controller(gesture::on_secondary_up({
            let window = window.clone();
            let steal_window = steal_window.clone();
            let modules = modules.clone();
            move |_, _, _| {
                // usually signifies that a module is being collapsed, but we should make sure that all are collapsed
                let any_expanded = modules.values().any(|wrapper| wrapper.module.is_expanded());
                if !any_expanded {
                    steal_window.set_visible(false);
                    window.set_layer(Layer::Top);
                    window.set_keyboard_mode(KeyboardMode::None);
                }
            }
        }));

        // the bar window should be above the steal window, we can assume any click here is outside the bar
        steal_window.add_controller(gesture::on_primary_up(move |_, _, _| {
            hide_all_expanded_modules();
        }));

        BarWindow {
            window,
            monitor: monitor.clone(),
            steal_window,
            modules,
        }
    }

    pub fn hide_all_expanded_modules(&self) {
        for wrapper in self.modules.values() {
            wrapper.module.set_expanded(false);
        }

        self.steal_window.set_visible(false);
        self.window.set_layer(Layer::Top);
        self.window.set_keyboard_mode(KeyboardMode::None);
    }
}

pub fn toggle_module_by_name(name: &str) {
    let Some(monitor) = hyprland::get_active_monitor() else {
        return;
    };

    APP_LOCAL.with(|app| {
        let app = app.borrow();
        let bar_windows = app.bars.borrow();
        for bar_window in &*bar_windows {
            if bar_window.monitor == monitor {
                if let Some(wrapper) = bar_window.modules.get(name) {
                    wrapper.module.set_expanded(!wrapper.module.is_expanded());

                    // manage steal window visibility
                    let any_expanded = bar_window.modules.values().any(|w| w.module.is_expanded());
                    if any_expanded {
                        bar_window.steal_window.set_visible(true);
                        bar_window.window.set_layer(Layer::Overlay);
                        bar_window.window.set_keyboard_mode(KeyboardMode::OnDemand);
                    } else {
                        bar_window.steal_window.set_visible(false);
                        bar_window.window.set_layer(Layer::Top);
                        bar_window.window.set_keyboard_mode(KeyboardMode::None);
                    }
                }
            }
        }
    });
}

pub fn listen_for_ipc_messages() {
    ipc::listen_for_messages_local(|message| {
        if let Some(module_name) = message.strip_prefix("toggle_bar_module_") {
            toggle_module_by_name(module_name);
        }
    });
}