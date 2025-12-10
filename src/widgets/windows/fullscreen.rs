use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::gesture;
use crate::singletons::hyprland;
use super::{GmsWindow, hide_all_fullscreen_windows};

/// A window that takes up the whole screen. It closes itself when it loses focus.
#[derive(Clone)]
pub struct FullscreenWindow {
    pub window: gtk4::ApplicationWindow,
}

impl GmsWindow for FullscreenWindow {
    fn show(&self) {
        hide_all_fullscreen_windows();
        let monitor = hyprland::get_active_monitor();
        self.window.set_monitor(monitor.as_ref());
        self.window.show();
    }

    fn hide(&self) {
        self.window.hide();
    }

    fn toggle(&self) -> bool {
        let was_visible = self.is_visible();
        if was_visible {
            self.hide();
        } else {
            self.show();
        }
        !was_visible
    }

    fn is_visible(&self) -> bool {
        self.window.is_visible()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl FullscreenWindow {
    pub fn new(
        application: &libadwaita::Application,
        classes: &[&str],
        child: &impl IsA<gtk4::Widget>
    ) -> Self {
        let monitor = hyprland::get_active_monitor();
        let window = gtk4::ApplicationWindow::new(application);
        
        window.set_css_classes(classes);
        window.init_layer_shell();
        window.set_monitor(monitor.as_ref());
        window.set_keyboard_mode(KeyboardMode::OnDemand);
        window.set_layer(Layer::Top);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Bottom, true);
        window.set_namespace(Some("gms-fullscreen"));
        window.set_child(Some(child));

        let fullscreen = Self {
            window: window.clone(),
        };

        window.add_controller(gesture::on_primary_full_press({
            let window = window.clone();
            let fullscreen = fullscreen.clone();

            move |_, p_xy, r_xy| {
                let allocation = window.child().unwrap().allocation();
                let (px, py) = (p_xy.0 as i32, p_xy.1 as i32);
                let (rx, ry) = (r_xy.0 as i32, r_xy.1 as i32);

                if window.is_visible() && !allocation.contains_point(px, py) && !allocation.contains_point(rx, ry) {
                    fullscreen.hide();
                }
            }
        }));

        window.add_controller(gesture::on_key_press({
            let fullscreen = fullscreen.clone();

            move |val, _| {
                if val.name() == Some("Escape".into()) {
                    fullscreen.hide();
                }
            }
        }));

        fullscreen
    }
}