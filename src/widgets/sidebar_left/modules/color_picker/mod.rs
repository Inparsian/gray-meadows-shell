mod hue_picker;
mod saturation_value_picker;
mod fields;

use futures_signals::signal::{Mutable, SignalExt};
use gtk4::prelude::*;

use crate::{color::model::{int_to_hex, Hsv}, widgets::{common::tabs::{TabSize, Tabs, TabsStack}, sidebar_left::modules::color_picker::fields::Fields}};

pub fn new() -> gtk4::Box {
    let hsv = Mutable::new(Hsv {
        hue: 0.0,
        saturation: 100.0,
        value: 100.0
    });

    let hue_picker = hue_picker::HuePicker::new(&hsv);
    let saturation_value_picker = saturation_value_picker::SaturationValuePicker::new(&hsv);

    let tabs = Tabs::new(TabSize::Normal, false);
    tabs.current_tab.set(Some("hsv".to_owned()));
    tabs.add_tab("HEX", "hex".to_owned(), None);
    tabs.add_tab("INT", "int".to_owned(), None);
    tabs.add_tab("RGB", "rgb".to_owned(), None);
    tabs.add_tab("HSV", "hsv".to_owned(), None);
    tabs.add_tab("HSL", "hsl".to_owned(), None);
    tabs.add_tab("CMYK", "cmyk".to_owned(), None);
    tabs.add_tab("OKLCH", "oklch".to_owned(), None);

    let tabs_stack = TabsStack::new(&tabs, Some("color-picker-tabs-stack"));

    // !! These will be separated into their own modules later
    let mut hex_fields = Fields::new();
    hex_fields.add_field(fields::FieldType::Entry, {
        let hsv = hsv.clone();
        move |result| {
            if let fields::FieldUpdate::Text(hex) = result {
                hsv.set(Hsv::from_hex(&hex));
            }
        }
    });

    let mut int_fields = Fields::new();
    int_fields.add_field(fields::FieldType::Entry, {
        let hsv = hsv.clone();
        move |result| {
            if let fields::FieldUpdate::Text(int) = result {
                let hex = int_to_hex(int.parse().unwrap_or(0));
                hsv.set(Hsv::from_hex(&hex));
            }
        }
    });

    let mut rgb_fields = Fields::new();
    rgb_fields.add_field(fields::FieldType::SpinButton(0, 1.0, gtk4::Adjustment::new(0.0, 0.0, 255.0, 1.0, 1.0, 0.0)), {
        let hsv = hsv.clone();
        move |result| {
            if let fields::FieldUpdate::Float(value) = result {
                let mut rgba = hsv.get().as_rgba();
                rgba.red = value as u8;
                hsv.set(Hsv::from_hex(&rgba.as_hex()));
            }
        }
    });

    rgb_fields.add_field(fields::FieldType::SpinButton(0, 1.0, gtk4::Adjustment::new(0.0, 0.0, 255.0, 1.0, 1.0, 0.0)), {
        let hsv = hsv.clone();
        move |result| {
            if let fields::FieldUpdate::Float(value) = result {
                let mut rgba = hsv.get().as_rgba();
                rgba.green = value as u8;
                hsv.set(Hsv::from_hex(&rgba.as_hex()));
            }
        }
    });

    rgb_fields.add_field(fields::FieldType::SpinButton(0, 1.0, gtk4::Adjustment::new(0.0, 0.0, 255.0, 1.0, 1.0, 0.0)), {
        let hsv = hsv.clone();
        move |result| {
            if let fields::FieldUpdate::Float(value) = result {
                let mut rgba = hsv.get().as_rgba();
                rgba.blue = value as u8;
                hsv.set(Hsv::from_hex(&rgba.as_hex()));
            }
        }
    });

    let mut hsv_fields = Fields::new();
    hsv_fields.add_field(fields::FieldType::SpinButton(2, 0.33, gtk4::Adjustment::new(0.0, 0.0, 360.0, 0.33, 0.33, 0.0)), {
        let hsv = hsv.clone();
        move |result| {
            if let fields::FieldUpdate::Float(value) = result {
                let mut hsv_value = hsv.get();
                hsv_value.hue = value;
                hsv.set(hsv_value);
            }
        }
    });

    hsv_fields.add_field(fields::FieldType::SpinButton(2, 0.33, gtk4::Adjustment::new(0.0, 0.0, 100.0, 0.33, 0.33, 0.0)), {
        let hsv = hsv.clone();
        move |result| {
            if let fields::FieldUpdate::Float(value) = result {
                let mut hsv_value = hsv.get();
                hsv_value.saturation = value;
                hsv.set(hsv_value);
            }
        }
    });

    hsv_fields.add_field(fields::FieldType::SpinButton(2, 0.33, gtk4::Adjustment::new(0.0, 0.0, 100.0, 0.33, 0.33, 0.0)), {
        let hsv = hsv.clone();
        move |result| {
            if let fields::FieldUpdate::Float(value) = result {
                let mut hsv_value = hsv.get();
                hsv_value.value = value;
                hsv.set(hsv_value);
            }
        }
    });

    let mut hsl_fields = Fields::new();
    hsl_fields.add_field(fields::FieldType::SpinButton(2, 0.33, gtk4::Adjustment::new(0.0, 0.0, 360.0, 0.33, 0.33, 0.0)), {
        let hsv = hsv.clone();
        move |result| {
            if let fields::FieldUpdate::Float(value) = result {
                let mut hsl_value = hsv.get().as_hsl();
                hsl_value.hue = value;
                hsv.set(Hsv::from_hex(&hsl_value.as_hex()));
            }
        }
    });

    hsl_fields.add_field(fields::FieldType::SpinButton(2, 0.33, gtk4::Adjustment::new(0.0, 0.0, 100.0, 0.33, 0.33, 0.0)), {
        let hsv = hsv.clone();
        move |result| {
            if let fields::FieldUpdate::Float(value) = result {
                let mut hsl_value = hsv.get().as_hsl();
                hsl_value.saturation = value;
                hsv.set(Hsv::from_hex(&hsl_value.as_hex()));
            }
        }
    });

    hsl_fields.add_field(fields::FieldType::SpinButton(2, 0.33, gtk4::Adjustment::new(0.0, 0.0, 100.0, 0.33, 0.33, 0.0)), {
        let hsv = hsv.clone();
        move |result| {
            if let fields::FieldUpdate::Float(value) = result {
                let mut hsl_value = hsv.get().as_hsl();
                hsl_value.lightness = value;
                hsv.set(Hsv::from_hex(&hsl_value.as_hex()));
            }
        }
    });

    let mut cmyk_fields = Fields::new();
    cmyk_fields.add_field(fields::FieldType::SpinButton(0, 1.0, gtk4::Adjustment::new(0.0, 0.0, 100.0, 1.0, 1.0, 0.0)), {
        let hsv = hsv.clone();
        move |result| {
            if let fields::FieldUpdate::Float(value) = result {
                let mut cmyk_value = hsv.get().as_cmyk();
                cmyk_value.cyan = value as u8;
                hsv.set(Hsv::from_hex(&cmyk_value.as_hex()));
            }
        }
    });

    cmyk_fields.add_field(fields::FieldType::SpinButton(0, 1.0, gtk4::Adjustment::new(0.0, 0.0, 100.0, 1.0, 1.0, 0.0)), {
        let hsv = hsv.clone();
        move |result| {
            if let fields::FieldUpdate::Float(value) = result {
                let mut cmyk_value = hsv.get().as_cmyk();
                cmyk_value.magenta = value as u8;
                hsv.set(Hsv::from_hex(&cmyk_value.as_hex()));
            }
        }
    });

    cmyk_fields.add_field(fields::FieldType::SpinButton(0, 1.0, gtk4::Adjustment::new(0.0, 0.0, 100.0, 1.0, 1.0, 0.0)), {
        let hsv = hsv.clone();
        move |result| {
            if let fields::FieldUpdate::Float(value) = result {
                let mut cmyk_value = hsv.get().as_cmyk();
                cmyk_value.yellow = value as u8;
                hsv.set(Hsv::from_hex(&cmyk_value.as_hex()));
            }
        }
    });

    cmyk_fields.add_field(fields::FieldType::SpinButton(0, 1.0, gtk4::Adjustment::new(0.0, 0.0, 100.0, 1.0, 1.0, 0.0)), {
        let hsv = hsv.clone();
        move |result| {
            if let fields::FieldUpdate::Float(value) = result {
                let mut cmyk_value = hsv.get().as_cmyk();
                cmyk_value.black = value as u8;
                hsv.set(Hsv::from_hex(&cmyk_value.as_hex()));
            }
        }
    });

    let mut oklch_fields = Fields::new();
    oklch_fields.add_field(fields::FieldType::SpinButton(4, 0.033, gtk4::Adjustment::new(0.0, 0.0, 100.0, 0.033, 0.033, 0.0)), {
        let hsv = hsv.clone();
        move |result| {
            if let fields::FieldUpdate::Float(value) = result {
                let mut oklch_value = hsv.get().as_oklch();
                oklch_value.lightness = value;
                hsv.set(Hsv::from_hex(&oklch_value.as_hex()));
            }
        }
    });

    oklch_fields.add_field(fields::FieldType::SpinButton(4, 0.033, gtk4::Adjustment::new(0.0, 0.0, 100.0, 0.033, 0.033, 0.0)), {
        let hsv = hsv.clone();
        move |result| {
            if let fields::FieldUpdate::Float(value) = result {
                let mut oklch_value = hsv.get().as_oklch();
                oklch_value.chroma = value;
                hsv.set(Hsv::from_hex(&oklch_value.as_hex()));
            }
        }
    });

    oklch_fields.add_field(fields::FieldType::SpinButton(2, 0.33, gtk4::Adjustment::new(0.0, 0.0, 360.0, 0.33, 0.33, 0.0)), {
        let hsv = hsv.clone();
        move |result| {
            if let fields::FieldUpdate::Float(value) = result {
                let mut oklch_value = hsv.get().as_oklch();
                oklch_value.hue = value;
                hsv.set(Hsv::from_hex(&oklch_value.as_hex()));
            }
        }
    });

    view! {
        widget = gtk4::Box {
            set_css_classes: &["ColorPicker"],
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 12,
            set_hexpand: true,
            set_vexpand: true,

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 8,

                append: hue_picker.get_widget(),
                append: saturation_value_picker.get_widget()
            },

            append: &tabs.widget,
            append: &tabs_stack.widget
        }
    }

    tabs_stack.add_tab(Some("hex"), &hex_fields.widget);
    tabs_stack.add_tab(Some("int"), &int_fields.widget);
    tabs_stack.add_tab(Some("rgb"), &rgb_fields.widget);
    tabs_stack.add_tab(Some("hsv"), &hsv_fields.widget);
    tabs_stack.add_tab(Some("hsl"), &hsl_fields.widget);
    tabs_stack.add_tab(Some("cmyk"), &cmyk_fields.widget);
    tabs_stack.add_tab(Some("oklch"), &oklch_fields.widget);

    let hsv_future = hsv.signal().for_each(move |hsv| {
        let hex = hsv.as_hex();
        let int = hsv.as_int();
        let rgba = hsv.as_rgba();
        let hsl = hsv.as_hsl();
        let cmyk = hsv.as_cmyk();
        let oklch = hsv.as_oklch();

        hex_fields.update(vec![fields::FieldUpdate::Text(hex)]);
        int_fields.update(vec![fields::FieldUpdate::Text(int.to_string())]);
        rgb_fields.update(vec![
            fields::FieldUpdate::Float(rgba.red as f64), 
            fields::FieldUpdate::Float(rgba.green as f64),
            fields::FieldUpdate::Float(rgba.blue as f64)
        ]);
        hsv_fields.update(vec![
            fields::FieldUpdate::Float(hsv.hue),
            fields::FieldUpdate::Float(hsv.saturation),
            fields::FieldUpdate::Float(hsv.value)
        ]);
        hsl_fields.update(vec![
            fields::FieldUpdate::Float(hsl.hue),
            fields::FieldUpdate::Float(hsl.saturation),
            fields::FieldUpdate::Float(hsl.lightness)
        ]);
        cmyk_fields.update(vec![
            fields::FieldUpdate::Float(cmyk.cyan as f64),
            fields::FieldUpdate::Float(cmyk.magenta as f64),
            fields::FieldUpdate::Float(cmyk.yellow as f64),
            fields::FieldUpdate::Float(cmyk.black as f64)
        ]);
        oklch_fields.update(vec![
            fields::FieldUpdate::Float(oklch.lightness),
            fields::FieldUpdate::Float(oklch.chroma),
            fields::FieldUpdate::Float(oklch.hue)
        ]);

        async {}
    });

    gtk4::glib::spawn_future_local(hsv_future);

    widget
}