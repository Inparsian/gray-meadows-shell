pub mod base;
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
use crate::ipc;
use crate::singletons::hyprland;
use crate::utils::gesture;
use self::base::{BarModule, hide_all_expanded_modules};

static BAR_HEIGHT: i32 = 33;

pub struct BarWindow {
    pub window: gtk4::ApplicationWindow,
    pub monitor: gdk4::Monitor,
    pub steal_window: gtk4::ApplicationWindow,
    pub modules: HashMap<String, BarModule>,
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

                append: &sysstats_module,
                append: &mpris_module,
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
                    set_start_widget: Some(&left_box),
                    set_center_widget: Some(&center_box),
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
        window.add_controller(gesture::on_primary_full_press(clone!(
            #[weak] window,
            #[weak] steal_window,
            #[strong] modules,
            move |_, (px, py), (rx, ry)| {
                if py > BAR_HEIGHT as f64 && ry > BAR_HEIGHT as f64 {
                    let not_inside_any = modules.values().filter(|module| module.expanded()).any(|module| {
                        let mod_allocation = module.allocation();
                        let parent_allocation = module.parent().expect("No parent for bar module").allocation();
                        let allocation = gdk4::Rectangle::new(
                            mod_allocation.x() + parent_allocation.x(),
                            mod_allocation.y() + parent_allocation.y(),
                            mod_allocation.width(),
                            mod_allocation.height(),
                        );
    
                        !allocation.contains_point(rx as i32, ry as i32)
                            && !allocation.contains_point(px as i32, py as i32)
                    });

                    if not_inside_any {
                        hide_all_expanded_modules();
                    }
                }

                if modules.values().any(|module| module.expanded()) {
                    steal_window.set_visible(true);
                    window.set_layer(Layer::Overlay);
                    window.set_keyboard_mode(KeyboardMode::OnDemand);
                }
            }
        )));

        window.add_controller(gesture::on_secondary_up(clone!(
            #[weak] window,
            #[weak] steal_window,
            #[strong] modules,
            move |_, _, _| {
                // usually signifies that a module is being collapsed, but we should make sure that all are collapsed
                let any_expanded = modules.values().any(|module| module.expanded());
                if !any_expanded {
                    steal_window.set_visible(false);
                    window.set_layer(Layer::Top);
                    window.set_keyboard_mode(KeyboardMode::None);
                }
            }
        )));

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
        for module in self.modules.values() {
            module.set_expanded(false);
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
        for bar_window in &*app.bars.borrow() {
            if bar_window.monitor == monitor && let Some(module) = bar_window.modules.get(name) {
                module.set_expanded(!module.expanded());

                let any_expanded = bar_window.modules.values().any(|module| module.expanded());
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
    });
}

pub fn listen_for_ipc_messages() {
    ipc::listen_for_messages_local(|message| {
        if let Some(module_name) = message.strip_prefix("toggle_bar_module_") {
            toggle_module_by_name(module_name);
        }
    });
}