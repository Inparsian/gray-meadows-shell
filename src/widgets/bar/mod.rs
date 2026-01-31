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

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell as _};

use crate::APP_LOCAL;
use crate::ipc;
use crate::services::hyprland;
use crate::utils::gesture;
use self::base::{BarModule, hide_all_expanded_modules};

static BAR_HEIGHT: i32 = 33;

#[derive(glib::Downgrade)]
pub struct BarWindow {
    pub window: gtk4::ApplicationWindow,
    pub monitor: gdk4::Monitor,
    pub steal_window: gtk4::ApplicationWindow,
    pub modules: Rc<RefCell<HashMap<String, BarModule>>>,
}

impl BarWindow {
    pub fn new(application: &libadwaita::Application, monitor: &gdk4::Monitor) -> Self {
        let mpris_module = modules::mpris::new();
        let sysstats_module = modules::sysstats::new();
        let modules = Rc::new(RefCell::new(HashMap::new()));
        modules.borrow_mut().insert("mpris".to_owned(), mpris_module.clone());
        modules.borrow_mut().insert("sysstats".to_owned(), sysstats_module.clone());

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
        
        let me = BarWindow {
            window,
            monitor: monitor.clone(),
            steal_window,
            modules,
        };

        // collapse expanded modules when clicking outside of them
        me.window.add_controller(gesture::on_primary_full_press(clone!(
            #[weak] me,
            move |_, (px, py), (rx, ry)| {
                if py > BAR_HEIGHT as f64 && ry > BAR_HEIGHT as f64 {
                    let not_inside_any = me.modules.borrow().values().filter(|module| module.expanded()).any(|module| {
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

                me.set_steal_window_visibility();
            }
        )));

        me.window.add_controller(gesture::on_secondary_up(clone!(
            #[weak] me,
            move |_, _, _| {
                // usually signifies that a module is being collapsed, but we should make sure that all are collapsed
                me.set_steal_window_visibility();
            }
        )));

        // the bar window should be above the steal window, we can assume any click here is outside the bar
        me.steal_window.add_controller(gesture::on_primary_up(move |_, _, _| {
            hide_all_expanded_modules();
        }));
        
        me
    }

    pub fn hide_all_expanded_modules(&self) {
        for module in self.modules.borrow().values() {
            module.set_expanded(false);
        }

        self.set_steal_window_visibility();
    }
    
    pub fn set_steal_window_visibility(&self) {
        glib::idle_add_local_once(clone!(
            #[weak(rename_to = me)] self,
            move || {
                let any_expanded = me.modules.borrow().values().any(|module| module.expanded());
                if any_expanded {
                    me.steal_window.set_visible(true);
                    me.window.set_layer(Layer::Overlay);
                    me.window.set_keyboard_mode(KeyboardMode::OnDemand);
                } else {
                    me.steal_window.set_visible(false);
                    me.window.set_layer(Layer::Top);
                    me.window.set_keyboard_mode(KeyboardMode::None);
                }
            }
        ));
    }
}

pub fn toggle_module_by_name(name: &str) {
    let Some(monitor) = hyprland::get_active_monitor() else {
        return;
    };

    APP_LOCAL.with(|app| {
        for bar_window in &*app.bars.borrow() {
            if bar_window.monitor == monitor && let Some(module) = bar_window.modules.borrow().get(name) {
                module.set_expanded(!module.expanded());
                bar_window.set_steal_window_visibility();
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