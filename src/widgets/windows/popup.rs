use gtk4::{prelude::*, RevealerTransitionType};
use gtk4_layer_shell::{Edge, Layer, KeyboardMode, LayerShell};
use libadwaita::Clamp;

use crate::gesture;
use crate::singletons::hyprland;
use super::{GmsWindow, hide_all_popups};

/// A popup window that displays content on top of other windows. It closes itself when it loses focus.
#[derive(Clone)]
pub struct PopupWindow {
    pub window: gtk4::ApplicationWindow,
    pub revealer: gtk4::Revealer,
    pub options: PopupOptions,
    pub transition_duration: u32,
}

/// A margin for a popup window.
#[derive(Debug, Clone, Copy)]
pub struct PopupMargin {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

/// Special options for a popup window.
#[derive(Debug, Clone, Copy)]
pub struct PopupOptions {
    pub anchor_left: bool,
    pub anchor_right: bool,
    pub anchor_top: bool,
    pub anchor_bottom: bool
}

impl GmsWindow for PopupWindow {
    fn show(&self) {
        hide_all_popups();
        let monitor = hyprland::get_active_monitor();
        self.window.set_monitor(monitor.as_ref());
        self.window.show();
        self.steal_screen();

        gtk4::glib::timeout_add_local_once(std::time::Duration::from_millis(10), {
            let revealer = self.revealer.clone();
            move || revealer.set_reveal_child(true)
        });
    }

    fn hide(&self) {
        self.revealer.set_reveal_child(false);
        self.release_screen();

        gtk4::glib::timeout_add_local_once(std::time::Duration::from_millis(self.transition_duration as u64), {
            let window = self.window.clone();
            let revealer = self.revealer.clone();
            move || if !revealer.reveals_child() {
                window.hide();
            }
        });
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
        self.revealer.reveals_child()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl PopupWindow {
    /// Creates a new popup window.
    #[allow(clippy::too_many_arguments)] // bruh
    pub fn new(
        application: &libadwaita::Application,
        classes: &[&str],
        child: &impl IsA<gtk4::Widget>,
        options: PopupOptions,
        width: i32,
        height: i32,
        margin: PopupMargin,
        transition_type: RevealerTransitionType,
        transition_duration: u32
    ) -> Self {
        let monitor = hyprland::get_active_monitor();
        let window = gtk4::ApplicationWindow::new(application);
        window.set_css_classes(classes);
        window.init_layer_shell();
        window.set_monitor(monitor.as_ref());
        window.set_keyboard_mode(KeyboardMode::OnDemand);
        window.set_layer(Layer::Overlay);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Bottom, true);
        window.set_namespace(Some("gms-popup"));

        let revealer = gtk4::Revealer::new();
        revealer.set_transition_type(transition_type);
        revealer.set_transition_duration(transition_duration);
        revealer.set_reveal_child(false);
        if options.anchor_left && !options.anchor_right {
            revealer.set_halign(gtk4::Align::Start);
        }
        else if options.anchor_right && !options.anchor_left {
            revealer.set_halign(gtk4::Align::End);
        }
        if options.anchor_top && !options.anchor_bottom {
            revealer.set_valign(gtk4::Align::Start);
        }
        else if options.anchor_bottom && !options.anchor_top {
            revealer.set_valign(gtk4::Align::End);
        }

        let clamp = Clamp::new();
        clamp.set_focusable(true);
        clamp.set_maximum_size(if options.anchor_left && options.anchor_right {
            -1
        } else {
            width
        });
        clamp.set_height_request(if options.anchor_top && options.anchor_bottom {
            -1
        } else {
            height
        });
        clamp.set_margin_top(margin.top);
        clamp.set_margin_end(margin.right);
        clamp.set_margin_bottom(margin.bottom);
        clamp.set_margin_start(margin.left);

        let container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        container.append(child);
        
        clamp.set_child(Some(&container));
        revealer.set_child(Some(&clamp));
        window.set_child(Some(&revealer));

        let popup = Self {
            window: window.clone(),
            revealer,
            options,
            transition_duration,
        };

        window.add_controller(gesture::on_key_press({
            let popup = popup.clone();

            move |val, _| if val.name() == Some("Escape".into()) {
                popup.hide();
            }
        }));

        window.add_controller(gesture::on_primary_full_press({
            let popup = popup.clone();

            move |_, p_xy, r_xy| {
                let allocation = clamp.allocation();
                let (px, py) = (p_xy.0 as i32, p_xy.1 as i32);
                let (rx, ry) = (r_xy.0 as i32, r_xy.1 as i32);

                if popup.window.is_visible() && !allocation.contains_point(px, py) && !allocation.contains_point(rx, ry) {
                    popup.hide();
                }
            }
        }));

        popup
    }

    pub fn steal_screen(&self) {
        self.window.set_anchor(Edge::Left, true);
        self.window.set_anchor(Edge::Right, true);
        self.window.set_anchor(Edge::Top, true);
        self.window.set_anchor(Edge::Bottom, true);
    }

    pub fn release_screen(&self) {
        self.window.set_anchor(Edge::Left, self.options.anchor_left);
        self.window.set_anchor(Edge::Right, self.options.anchor_right);
        self.window.set_anchor(Edge::Top, self.options.anchor_top);
        self.window.set_anchor(Edge::Bottom, self.options.anchor_bottom);
    }
}