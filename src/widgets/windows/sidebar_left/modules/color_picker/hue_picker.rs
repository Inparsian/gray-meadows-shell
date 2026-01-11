use std::{rc::Rc, cell::RefCell};
use futures_signals::signal::{Mutable, SignalExt as _};
use gtk4::prelude::*;

use crate::color::model::Hsv;
use crate::utils::gesture;

#[derive(Debug, Clone)]
pub struct HuePicker {
    pub hsv: Mutable<Hsv>,
    pub widget: gtk4::Box,
    pub trough: gtk4::Box,
    trough_css_provider: gtk4::CssProvider
}

impl HuePicker {
    pub fn new(hsv: &Mutable<Hsv>) -> Self {
        let trough_css_provider = gtk4::CssProvider::new();

        let trough = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        trough.set_css_classes(&["color-picker-hue-trough"]);

        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        widget.set_css_classes(&["color-picker-hue"]);
        widget.append(&trough);

        trough.style_context().add_provider(
            &trough_css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let picker = Self {
            hsv: hsv.clone(),
            widget: widget.clone(),
            trough,
            trough_css_provider
        };

        let clicked = Rc::new(RefCell::new(false));

        widget.add_controller(gesture::on_primary_down({
            let picker = picker.clone();
            let clicked = clicked.clone();

            move |_, _, y| {
                *clicked.borrow_mut() = true;
                picker.handle_click(y);
            }
        }));

        widget.add_controller(gesture::on_motion({
            let picker = picker.clone();
            let clicked = clicked.clone();

            move |_, y| if *clicked.borrow() {
                picker.handle_click(y);
            }
        }));

        widget.add_controller(gesture::on_primary_up(move |_, _, _| {
            *clicked.borrow_mut() = false;
        }));

        widget.add_controller(gesture::on_vertical_scroll({
            let picker = picker.clone();
            move |y| picker.handle_scroll(y * 5.0)
        }));

        gtk4::glib::spawn_future_local({
            let picker = picker.clone();
            signal!(hsv, (_) { picker.update_trough_position(); })
        });

        picker
    }

    pub fn handle_scroll(&self, y: f64) {
        let mut hue = (self.hsv.get().hue + y) % 360.0;
        if hue < 0.0 {
            hue += 360.0;
        }

        self.hsv.set(Hsv {
            hue,
            saturation: self.hsv.get().saturation,
            value: self.hsv.get().value,
        });
    }

    pub fn handle_click(&self, y: f64) {
        let clamped_y = y.clamp(0.0, self.widget.height() as f64);
        let hue = (clamped_y / self.widget.height() as f64) * 360.0;

        self.hsv.set(Hsv {
            hue,
            saturation: self.hsv.get().saturation,
            value: self.hsv.get().value,
        });
    }

    pub fn update_trough_position(&self) {
        let hue = self.hsv.get().hue;
        let widget_height = self.widget.height() as f64;
        let trough_height = self.trough.height() as f64;
        let trough_pos = ((widget_height - trough_height) * (hue / 360.0)).round() as i32;

        self.trough_css_provider.load_from_data(&format!("
            .color-picker-hue-trough {{
                margin-top: {trough_pos}px;
                margin-bottom: -{trough_pos}px;
            }}
        "));
    }

    pub fn get_widget(&self) -> &gtk4::Box {
        &self.widget
    }
}