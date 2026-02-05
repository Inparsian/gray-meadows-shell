use std::{rc::Rc, cell::RefCell};
use futures_signals::signal::{Mutable, SignalExt as _};
use gtk::prelude::*;

use crate::color::models::Hsv;
use crate::utils::gesture;

#[derive(Debug, Clone)]
pub struct HuePicker {
    pub hsv: Mutable<Hsv>,
    pub widget: gtk::Box,
    pub trough: gtk::Box,
    trough_css_provider: gtk::CssProvider
}

impl glib::clone::Downgrade for HuePicker {
    type Weak = HuePickerWeak;
    
    fn downgrade(&self) -> Self::Weak {
        HuePickerWeak {
            hsv: self.hsv.clone(),
            widget: glib::clone::Downgrade::downgrade(&self.widget),
            trough: glib::clone::Downgrade::downgrade(&self.trough),
            trough_css_provider: self.trough_css_provider.clone()
        }
    }
}

#[derive(Debug, Clone)]
pub struct HuePickerWeak {
    pub hsv: Mutable<Hsv>,
    pub widget: glib::WeakRef<gtk::Box>,
    pub trough: glib::WeakRef<gtk::Box>,
    trough_css_provider: gtk::CssProvider
}

impl glib::clone::Upgrade for HuePickerWeak {
    type Strong = HuePicker;
    
    fn upgrade(&self) -> Option<HuePicker> {
        Some(HuePicker {
            hsv: self.hsv.clone(),
            widget: self.widget.upgrade()?,
            trough: self.trough.upgrade()?,
            trough_css_provider: self.trough_css_provider.clone()
        })
    }
}

impl HuePicker {
    pub fn new(hsv: &Mutable<Hsv>) -> Self {
        let trough_css_provider = gtk::CssProvider::new();

        let trough = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        trough.set_css_classes(&["color-picker-hue-trough"]);

        let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
        widget.set_css_classes(&["color-picker-hue"]);
        widget.append(&trough);

        trough.style_context().add_provider(
            &trough_css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let picker = Self {
            hsv: hsv.clone(),
            widget: widget.clone(),
            trough,
            trough_css_provider
        };

        let clicked = Rc::new(RefCell::new(false));

        widget.add_controller(gesture::on_primary_down(clone!(
            #[weak] picker,
            #[weak] clicked,
            move |_, _, y| {
                *clicked.borrow_mut() = true;
                picker.handle_click(y);
            }
        )));

        widget.add_controller(gesture::on_motion(clone!(
            #[weak] picker,
            #[strong] clicked,
            move |_, y| if *clicked.borrow() {
                picker.handle_click(y);
            }
        )));

        widget.add_controller(gesture::on_primary_up(move |_, _, _| {
            *clicked.borrow_mut() = false;
        }));

        widget.add_controller(gesture::on_vertical_scroll(clone!(
            #[weak] picker,
            move |y| picker.handle_scroll(y * 5.0)
        )));

        glib::spawn_future_local({
            let picker = glib::clone::Downgrade::downgrade(&picker);
            signal!(hsv, (_) {
                if let Some(picker) = glib::clone::Upgrade::upgrade(&picker) {
                    picker.update_trough_position();
                }
            })
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

    pub fn get_widget(&self) -> &gtk::Box {
        &self.widget
    }
}