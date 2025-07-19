mod hue_picker;
mod saturation_value_picker;

use futures_signals::signal::{Mutable, SignalExt};
use gtk4::prelude::*;

use crate::color::model::Hsv;

pub fn new() -> gtk4::Box {
    let hsv = Mutable::new(Hsv {
        hue: 0.0,
        saturation: 100.0,
        value: 100.0
    });

    let hue_picker = hue_picker::HuePicker::new(&hsv);

    view! {
        test_hsv_label = gtk4::Label {
            set_text: {
                let hsv = hsv.get_cloned();
                &format!("HSV: {:.2}, {:.2}, {:.2}", hsv.hue, hsv.saturation, hsv.value)
            }
        },
        
        test_rgba_label = gtk4::Label {
            set_text: {
                let rgba = hsv.get_cloned().as_rgba();
                &format!("RGBA: {:.2}, {:.2}, {:.2}, {:.2}", rgba.red, rgba.green, rgba.blue, rgba.alpha)
            }
        },

        test_hex_label = gtk4::Label {
            set_text: {
                let hsv = hsv.get_cloned().as_hex();
                &format!("Hex: {}", hsv)
            }
        },

        widget = gtk4::Box {
            set_css_classes: &["ColorPicker"],
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 12,
            set_hexpand: true,
            set_vexpand: true,

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 8,

                append: hue_picker.get_widget()
            },

            append: &test_hsv_label,
            append: &test_rgba_label,
            append: &test_hex_label
        }
    }

    let hsv_future = hsv.signal().for_each(move |hsv| {
        let rgba = hsv.as_rgba();
        let hex = hsv.as_hex();

        test_hsv_label.set_text(&format!("HSV: {:.2}, {:.2}, {:.2}", hsv.hue, hsv.saturation, hsv.value));
        test_rgba_label.set_text(&format!("RGBA: {:.2}, {:.2}, {:.2}, {:.2}", rgba.red, rgba.green, rgba.blue, rgba.alpha));
        test_hex_label.set_text(&format!("Hex: {}", hex));

        async {}
    });

    gtk4::glib::spawn_future_local(hsv_future);

    widget
}