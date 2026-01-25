use gtk4::prelude::*;
use gtk4::cairo::{RectangleInt, Region};
use gtk4_layer_shell::{Edge, Layer, KeyboardMode, LayerShell as _};
use libadwaita::Clamp;

use crate::singletons::hyprland;
use crate::utils::gesture;
use crate::widgets::common::revealer::{AdwRevealer, AdwRevealerDirection, GEasing};
use super::{GmsWindow, hide_all_popups};

/// A popup window that displays content on top of other windows. It closes itself when it loses focus.
#[derive(Clone, glib::Downgrade)]
pub struct PopupWindow {
    pub window: gtk4::ApplicationWindow,
    pub revealer: AdwRevealer,
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
        self.set_clickthrough(false);
        self.window.add_css_class("visible");
        self.revealer.set_reveal(true);
    }

    fn hide(&self) {
        self.set_clickthrough(true);
        self.window.remove_css_class("visible");
        self.revealer.set_reveal(false);
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
        self.revealer.reveal()
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
        transition_direction: AdwRevealerDirection,
        transition_duration: u32
    ) -> Self {
        let window_classes = vec!["popup-window"]
            .into_iter()
            .chain(classes.iter().copied())
            .collect::<Vec<&str>>();

        let monitor = hyprland::get_active_monitor();
        let window = gtk4::ApplicationWindow::new(application);
        window.set_css_classes(&window_classes);
        window.init_layer_shell();
        window.set_monitor(monitor.as_ref());
        window.set_keyboard_mode(KeyboardMode::OnDemand);
        window.set_layer(Layer::Overlay);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Bottom, true);
        window.set_namespace(Some("gms-popup"));
        window.show();

        let revealer = AdwRevealer::builder()
            .css_classes(["popup-window-revealer"])
            .transition_duration(transition_duration)
            .transition_direction(transition_direction)
            .show_easing(GEasing::EaseOutExpo)
            .hide_easing(GEasing::EaseOutExpo)
            .build();
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
        clamp.set_unit(libadwaita::LengthUnit::Px);
        clamp.set_margin_top(margin.top);
        clamp.set_margin_end(margin.right);
        clamp.set_margin_bottom(margin.bottom);
        clamp.set_margin_start(margin.left);

        let container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        container.append(child);
        
        clamp.set_child(Some(&container));
        revealer.set_child(Some(&clamp.upcast()));
        window.set_child(Some(&revealer));

        let popup = Self {
            window,
            revealer,
        };

        popup.window.add_controller(gesture::on_key_press(clone!(
            #[weak] popup,
            move |key, _| if key.name() == Some("Escape".into()) {
                popup.hide();
            }
        )));

        popup.window.add_controller(gesture::on_primary_full_press(clone!(
            #[weak] popup,
            move |_, (px, py), (rx, ry)| {
                let allocation = popup.revealer.allocation();
                if popup.window.is_visible()
                    && !allocation.contains_point(px as i32, py as i32)
                    && !allocation.contains_point(rx as i32, ry as i32)
                {
                    popup.hide();
                }
            }
        )));

        popup.set_clickthrough(true);
        popup
    }

    pub fn set_clickthrough(&self, clickthrough: bool) {
        if let Some(surface) = self.window.native().and_then(|n| n.surface()) {
            let allocation = self.window.allocation();
            let region = if clickthrough {
                Region::create()
            } else {
                Region::create_rectangle(&RectangleInt::new(
                    allocation.x(),
                    allocation.y(),
                    allocation.width(),
                    allocation.height(),
                ))
            };
            surface.set_input_region(&region);
        }
    }
}