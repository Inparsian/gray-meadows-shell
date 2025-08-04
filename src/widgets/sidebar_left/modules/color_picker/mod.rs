mod hue_picker;
mod saturation_value_picker;
mod fields;

use futures_signals::signal::{Mutable, SignalExt};
use gtk4::prelude::*;

use crate::{color::model::{int_to_hex, Hsv}, ipc, widgets::{common::tabs::{TabSize, Tabs, TabsStack}, sidebar_left::modules::color_picker::fields::Fields}};

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
    macro_rules! create_entry_field {
        ($fields:ident, $hsv:ident, $convert:expr) => {
            $fields.add_field(fields::FieldType::Entry, {
                let hsv = $hsv.clone();
                move |result| {
                    if let fields::FieldUpdate::Text(value) = result {
                        hsv.set($convert(value));
                    }
                }
            });
        };
    }

    macro_rules! create_spin_field {
        ($fields:ident, $digits:expr, $step:expr, $adjustment:expr, $hsv:ident, $convert:expr) => {
            $fields.add_field(fields::FieldType::SpinButton($digits, $step, $adjustment), {
                let hsv = $hsv.clone();
                move |result: fields::FieldUpdate| {
                    if let fields::FieldUpdate::Float(value) = result {
                        hsv.set($convert(&hsv, value));
                    }
                }
            });
        };
    }

    let mut hex_fields = Fields::new();
    create_entry_field!(hex_fields, hsv, |hex: String| Hsv::from_hex(&hex));

    let mut int_fields = Fields::new();
    create_entry_field!(int_fields, hsv, |int: String| {
        let hex = int_to_hex(int.parse().unwrap_or(0));
        Hsv::from_hex(&hex)
    });

    let mut rgb_fields = Fields::new();
    create_spin_field!(rgb_fields, 0, 1.0, gtk4::Adjustment::new(0.0, 0.0, 255.0, 1.0, 1.0, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut rgba = hsv.get().as_rgba();
        rgba.red = value as u8;
        Hsv::from_hex(&rgba.as_hex())
    });
    create_spin_field!(rgb_fields, 0, 1.0, gtk4::Adjustment::new(0.0, 0.0, 255.0, 1.0, 1.0, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut rgba = hsv.get().as_rgba();
        rgba.green = value as u8;
        Hsv::from_hex(&rgba.as_hex())
    });
    create_spin_field!(rgb_fields, 0, 1.0, gtk4::Adjustment::new(0.0, 0.0, 255.0, 1.0, 1.0, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut rgba = hsv.get().as_rgba();
        rgba.blue = value as u8;
        Hsv::from_hex(&rgba.as_hex())
    });

    let mut hsv_fields = Fields::new();
    create_spin_field!(hsv_fields, 2, 0.33, gtk4::Adjustment::new(0.0, 0.0, 360.0, 0.33, 0.33, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut hsv_value = hsv.get();
        hsv_value.hue = value;
        hsv_value
    });
    create_spin_field!(hsv_fields, 2, 0.33, gtk4::Adjustment::new(0.0, 0.0, 100.0, 0.33, 0.33, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut hsv_value = hsv.get();
        hsv_value.saturation = value;
        hsv_value
    });
    create_spin_field!(hsv_fields, 2, 0.33, gtk4::Adjustment::new(0.0, 0.0, 100.0, 0.33, 0.33, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut hsv_value = hsv.get();
        hsv_value.value = value;
        hsv_value
    });

    let mut hsl_fields = Fields::new();
    create_spin_field!(hsl_fields, 2, 0.33, gtk4::Adjustment::new(0.0, 0.0, 360.0, 0.33, 0.33, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut hsl_value = hsv.get().as_hsl();
        hsl_value.hue = value;
        Hsv::from_hex(&hsl_value.as_hex())
    });
    create_spin_field!(hsl_fields, 2, 0.33, gtk4::Adjustment::new(0.0, 0.0, 100.0, 0.33, 0.33, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut hsl_value = hsv.get().as_hsl();
        hsl_value.saturation = value;
        Hsv::from_hex(&hsl_value.as_hex())
    });
    create_spin_field!(hsl_fields, 2, 0.33, gtk4::Adjustment::new(0.0, 0.0, 100.0, 0.33, 0.33, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut hsl_value = hsv.get().as_hsl();
        hsl_value.lightness = value;
        Hsv::from_hex(&hsl_value.as_hex())
    });

    let mut cmyk_fields = Fields::new();
    create_spin_field!(cmyk_fields, 0, 1.0, gtk4::Adjustment::new(0.0, 0.0, 100.0, 1.0, 1.0, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut cmyk_value = hsv.get().as_cmyk();
        cmyk_value.cyan = value as u8;
        Hsv::from_hex(&cmyk_value.as_hex())
    });
    create_spin_field!(cmyk_fields, 0, 1.0, gtk4::Adjustment::new(0.0, 0.0, 100.0, 1.0, 1.0, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut cmyk_value = hsv.get().as_cmyk();
        cmyk_value.magenta = value as u8;
        Hsv::from_hex(&cmyk_value.as_hex())
    });
    create_spin_field!(cmyk_fields, 0, 1.0, gtk4::Adjustment::new(0.0, 0.0, 100.0, 1.0, 1.0, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut cmyk_value = hsv.get().as_cmyk();
        cmyk_value.yellow = value as u8;
        Hsv::from_hex(&cmyk_value.as_hex())
    });
    create_spin_field!(cmyk_fields, 0, 1.0, gtk4::Adjustment::new(0.0, 0.0, 100.0, 1.0, 1.0, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut cmyk_value = hsv.get().as_cmyk();
        cmyk_value.black = value as u8;
        Hsv::from_hex(&cmyk_value.as_hex())
    });

    let mut oklch_fields = Fields::new();
    create_spin_field!(oklch_fields, 4, 0.033, gtk4::Adjustment::new(0.0, 0.0, 100.0, 0.033, 0.033, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut oklch_value = hsv.get().as_oklch();
        oklch_value.lightness = value;
        Hsv::from_hex(&oklch_value.as_hex())
    });
    create_spin_field!(oklch_fields, 4, 0.033, gtk4::Adjustment::new(0.0, 0.0, 100.0, 0.033, 0.033, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut oklch_value = hsv.get().as_oklch();
        oklch_value.chroma = value;
        Hsv::from_hex(&oklch_value.as_hex())
    });
    create_spin_field!(oklch_fields, 2, 0.33, gtk4::Adjustment::new(0.0, 0.0, 360.0, 0.33, 0.33, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut oklch_value = hsv.get().as_oklch();
        oklch_value.hue = value;
        Hsv::from_hex(&oklch_value.as_hex())
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
        use fields::FieldUpdate::*;

        let hex = hsv.as_hex();
        let int = hsv.as_int();
        let rgba = hsv.as_rgba();
        let hsl = hsv.as_hsl();
        let cmyk = hsv.as_cmyk();
        let oklch = hsv.as_oklch();

        hex_fields.update(vec![Text(hex)]);
        int_fields.update(vec![Text(int.to_string())]);
        rgb_fields.update(vec![Float(rgba.red as f64), Float(rgba.green as f64), Float(rgba.blue as f64)]);
        hsv_fields.update(vec![Float(hsv.hue), Float(hsv.saturation), Float(hsv.value)]);
        hsl_fields.update(vec![Float(hsl.hue), Float(hsl.saturation), Float(hsl.lightness)]);
        cmyk_fields.update(vec![Float(cmyk.cyan as f64), Float(cmyk.magenta as f64), Float(cmyk.yellow as f64), Float(cmyk.black as f64)]);
        oklch_fields.update(vec![Float(oklch.lightness), Float(oklch.chroma), Float(oklch.hue)]);

        async {}
    });

    // Listen for IPC messages to update the HSV value
    ipc::listen_for_messages_local(move |message| {
        let mut split_whitespace_iterator = message.split_whitespace();
        if let Some(message) = split_whitespace_iterator.next() {
            if message == "color_picker_set_hex" {
                if let Some(hex) = split_whitespace_iterator.next() {
                    let hsv_value = Hsv::from_hex(hex);

                    hsv.set(hsv_value);
                }
            }
        }
    });

    gtk4::glib::spawn_future_local(hsv_future);

    widget
}