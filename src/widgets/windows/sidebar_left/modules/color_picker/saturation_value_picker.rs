use std::{cell::RefCell, rc::Rc};
use futures_signals::signal::{Mutable, SignalExt};
use gtk4::prelude::*;

use crate::color::model::Hsv;
use crate::gesture;

#[derive(Debug, Clone)]
pub struct SaturationValuePicker {
    pub hsv: Mutable<Hsv>,
    pub widget: gtk4::Box,
    pub trough: gtk4::Box,
    widget_css_provider: gtk4::CssProvider,
    trough_css_provider: gtk4::CssProvider
}

impl SaturationValuePicker {
    pub fn new(hsv: &Mutable<Hsv>) -> Self {
        let widget_css_provider = gtk4::CssProvider::new();
        let trough_css_provider = gtk4::CssProvider::new();

        let trough = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        trough.set_css_classes(&["color-picker-saturation-value-trough"]);
        trough.set_halign(gtk4::Align::Start);
        trough.set_valign(gtk4::Align::Start);

        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        widget.set_css_classes(&["color-picker-saturation-value"]);
        widget.append(&trough);

        widget.style_context().add_provider(
            &widget_css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        trough.style_context().add_provider(
            &trough_css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let picker = Self {
            hsv: hsv.clone(),
            widget: widget.clone(),
            trough,
            widget_css_provider,
            trough_css_provider
        };

        let clicked = Rc::new(RefCell::new(false));

        widget.add_controller(gesture::on_primary_down({
            let picker = picker.clone();
            let clicked = clicked.clone();

            move |_, x, y| {
                *clicked.borrow_mut() = true;
                picker.handle_click(x, y);
            }
        }));

        widget.add_controller(gesture::on_motion({
            let picker = picker.clone();
            let clicked = clicked.clone();

            move |x, y| if *clicked.borrow() {
                picker.handle_click(x, y);
            }
        }));

        widget.add_controller(gesture::on_primary_up(move |_, _, _| {
            *clicked.borrow_mut() = false;
        }));

        gtk4::glib::spawn_future_local({
            let picker = picker.clone();
            signal!(hsv, (_) {
                picker.update_background_hue();
                picker.update_trough();
            })
        });

        picker
    }

    pub fn handle_click(&self, x: f64, y: f64) {
        let clamped_x = x.clamp(0.0, self.widget.width() as f64);
        let clamped_y = y.clamp(0.0, self.widget.height() as f64);
        let saturation = (clamped_x / self.widget.width() as f64) * 100.0;
        let value = (clamped_y / self.widget.height() as f64).mul_add(-100.0, 100.0);

        self.hsv.set(Hsv {
            hue: self.hsv.get().hue,
            saturation,
            value,
        });
    }

    pub fn update_background_hue(&self) {
        let hex = Hsv {
            hue: self.hsv.get().hue,
            saturation: 100.0,
            value: 100.0
        }.as_hex();

        self.widget_css_provider.load_from_data(&format!("
            .color-picker-saturation-value {{
                background: linear-gradient(to bottom, rgba(0,0,0,0), rgba(0,0,0,1)), linear-gradient(to right, #ffffff, {hex});
            }}
        "));
    }

    pub fn update_trough(&self) {
        let saturation = self.hsv.get().saturation;
        let value = self.hsv.get().value;

        let (widget_width, widget_height) = (self.widget.width() as f64, self.widget.height() as f64);
        let (trough_width, trough_height) = (self.trough.width() as f64, self.trough.height() as f64);

        let trough_pos_x = widget_width.mul_add(saturation / 100.0, -(trough_width / 2.0)).round() as i32;
        let trough_pos_y = (widget_height.mul_add(-(value / 100.0), widget_height) - (trough_height / 2.0)).round() as i32;

        self.trough_css_provider.load_from_data(&format!("
            .color-picker-saturation-value-trough {{
                margin-left: {}px;
                margin-right: {}px;
                margin-top: {}px;
                margin-bottom: {}px;
                border-color: {};
                background: {};
            }}",
            trough_pos_x,
            if trough_pos_x < 0 { trough_pos_x.abs() } else { -trough_pos_x },
            trough_pos_y,
            if trough_pos_y < 0 { trough_pos_y.abs() } else { -trough_pos_y },
            if value < 50.0 { "#ffffff" } else { "#000000" },
            self.hsv.get().as_hex()
        ));
    }

    pub fn get_widget(&self) -> &gtk4::Box {
        &self.widget
    }
}