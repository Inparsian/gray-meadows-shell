mod hue_picker;
mod saturation_value_picker;
mod fields;
mod color_boxes;

use std::time::Duration;
use futures_signals::signal::{Mutable, SignalExt as _};
use gtk4::{Adjustment, prelude::*};
use relm4::RelmIterChildrenExt as _;

use crate::color::parse_color_into_hex;
use crate::color::model::{int_to_hex, Hsv};
use crate::ipc;
use crate::singletons::clipboard;
use crate::utils::timeout::Timeout;
use crate::widgets::common::tabs::{TabSize, Tabs, TabsStack};
use self::fields::Fields;
use self::{saturation_value_picker::SaturationValuePicker, hue_picker::HuePicker};

pub fn new() -> gtk4::Box {
    let hsv = Mutable::new(Hsv {
        hue: 0.0,
        saturation: 100.0,
        value: 100.0
    });

    let hue_picker = HuePicker::new(&hsv);
    let saturation_value_picker = SaturationValuePicker::new(&hsv);

    let color_tabs = Tabs::new(TabSize::Normal, false);
    color_tabs.current_tab.set(Some("hsv".to_owned()));
    color_tabs.add_tab("HEX", "hex".to_owned(), None);
    color_tabs.add_tab("INT", "int".to_owned(), None);
    color_tabs.add_tab("RGB", "rgb".to_owned(), None);
    color_tabs.add_tab("HSV", "hsv".to_owned(), None);
    color_tabs.add_tab("HSL", "hsl".to_owned(), None);
    color_tabs.add_tab("CMYK", "cmyk".to_owned(), None);
    color_tabs.add_tab("OKLCH", "oklch".to_owned(), None);

    let transform_tabs = Tabs::new(TabSize::Normal, false);
    transform_tabs.current_tab.set(Some("analogous".to_owned()));
    transform_tabs.add_tab("ANALOGOUS", "analogous".to_owned(), None);
    transform_tabs.add_tab("TRIADIC", "triadic".to_owned(), None);
    transform_tabs.add_tab("TETRADIC", "tetradic".to_owned(), None);
    transform_tabs.add_tab("LIGHTNESS", "lighter_darker".to_owned(), None);

    let color_tabs_stack = TabsStack::new(&color_tabs, Some("color-picker-tabs-stack"));
    let transform_tabs_stack = TabsStack::new(&transform_tabs, Some("color-picker-tabs-stack"));

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
    create_spin_field!(rgb_fields, 0, 1.0, Adjustment::new(0.0, 0.0, 255.0, 1.0, 1.0, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut rgba = hsv.get().as_rgba();
        rgba.red = value as u8;
        Hsv::from_hex(&rgba.as_hex())
    });
    create_spin_field!(rgb_fields, 0, 1.0, Adjustment::new(0.0, 0.0, 255.0, 1.0, 1.0, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut rgba = hsv.get().as_rgba();
        rgba.green = value as u8;
        Hsv::from_hex(&rgba.as_hex())
    });
    create_spin_field!(rgb_fields, 0, 1.0, Adjustment::new(0.0, 0.0, 255.0, 1.0, 1.0, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut rgba = hsv.get().as_rgba();
        rgba.blue = value as u8;
        Hsv::from_hex(&rgba.as_hex())
    });

    let mut hsv_fields = Fields::new();
    create_spin_field!(hsv_fields, 2, 0.33, Adjustment::new(0.0, 0.0, 360.0, 0.33, 0.33, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut hsv_value = hsv.get();
        hsv_value.hue = value;
        hsv_value
    });
    create_spin_field!(hsv_fields, 2, 0.33, Adjustment::new(0.0, 0.0, 100.0, 0.33, 0.33, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut hsv_value = hsv.get();
        hsv_value.saturation = value;
        hsv_value
    });
    create_spin_field!(hsv_fields, 2, 0.33, Adjustment::new(0.0, 0.0, 100.0, 0.33, 0.33, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut hsv_value = hsv.get();
        hsv_value.value = value;
        hsv_value
    });

    let mut hsl_fields = Fields::new();
    create_spin_field!(hsl_fields, 2, 0.33, Adjustment::new(0.0, 0.0, 360.0, 0.33, 0.33, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut hsl_value = hsv.get().as_hsl();
        hsl_value.hue = value;
        Hsv::from_hex(&hsl_value.as_hex())
    });
    create_spin_field!(hsl_fields, 2, 0.33, Adjustment::new(0.0, 0.0, 100.0, 0.33, 0.33, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut hsl_value = hsv.get().as_hsl();
        hsl_value.saturation = value;
        Hsv::from_hex(&hsl_value.as_hex())
    });
    create_spin_field!(hsl_fields, 2, 0.33, Adjustment::new(0.0, 0.0, 100.0, 0.33, 0.33, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut hsl_value = hsv.get().as_hsl();
        hsl_value.lightness = value;
        Hsv::from_hex(&hsl_value.as_hex())
    });

    let mut cmyk_fields = Fields::new();
    create_spin_field!(cmyk_fields, 0, 1.0, Adjustment::new(0.0, 0.0, 100.0, 1.0, 1.0, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut cmyk_value = hsv.get().as_cmyk();
        cmyk_value.cyan = value as u8;
        Hsv::from_hex(&cmyk_value.as_hex())
    });
    create_spin_field!(cmyk_fields, 0, 1.0, Adjustment::new(0.0, 0.0, 100.0, 1.0, 1.0, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut cmyk_value = hsv.get().as_cmyk();
        cmyk_value.magenta = value as u8;
        Hsv::from_hex(&cmyk_value.as_hex())
    });
    create_spin_field!(cmyk_fields, 0, 1.0, Adjustment::new(0.0, 0.0, 100.0, 1.0, 1.0, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut cmyk_value = hsv.get().as_cmyk();
        cmyk_value.yellow = value as u8;
        Hsv::from_hex(&cmyk_value.as_hex())
    });
    create_spin_field!(cmyk_fields, 0, 1.0, Adjustment::new(0.0, 0.0, 100.0, 1.0, 1.0, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut cmyk_value = hsv.get().as_cmyk();
        cmyk_value.black = value as u8;
        Hsv::from_hex(&cmyk_value.as_hex())
    });

    let mut oklch_fields = Fields::new();
    create_spin_field!(oklch_fields, 4, 0.033, Adjustment::new(0.0, 0.0, 100.0, 0.033, 0.033, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut oklch_value = hsv.get().as_oklch();
        oklch_value.lightness = value;
        Hsv::from_hex(&oklch_value.as_hex())
    });
    create_spin_field!(oklch_fields, 4, 0.033, Adjustment::new(0.0, 0.0, 100.0, 0.033, 0.033, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut oklch_value = hsv.get().as_oklch();
        oklch_value.chroma = value;
        Hsv::from_hex(&oklch_value.as_hex())
    });
    create_spin_field!(oklch_fields, 2, 0.33, Adjustment::new(0.0, 0.0, 360.0, 0.33, 0.33, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut oklch_value = hsv.get().as_oklch();
        oklch_value.hue = value;
        Hsv::from_hex(&oklch_value.as_hex())
    });

    view! {
        paste_from_clipboard_button = gtk4::Button {
            set_label: "Paste from Clipboard",
            set_css_classes: &["color-picker-button"],

            gtk4::Box {
                set_orientation: gtk4::Orientation::Horizontal,
                set_spacing: 6,
                set_halign: gtk4::Align::Center,

                gtk4::Label {
                    set_css_classes: &["material-icons"],
                    set_label: "content_paste",
                },

                gtk4::Label {
                    set_widget_name: "paste-from-clipboard-label",
                    set_label: "Paste from Clipboard",
                }
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

                append: hue_picker.get_widget(),
                append: saturation_value_picker.get_widget()
            },

            append: &paste_from_clipboard_button,
            append: &color_tabs.widget,
            append: &color_tabs_stack.widget,
            append: &transform_tabs.widget,
            append: &transform_tabs_stack.widget
        }
    }

    let button_label_change_timeout = Timeout::default();
    paste_from_clipboard_button.connect_clicked({
        let hsv = hsv.clone();
        move |button| {
            let Some(clipboard_text) = clipboard::fetch_text_clipboard() else {
                return;
            };
        
            if let Some(hex) = parse_color_into_hex(&clipboard_text) {
                let hsv_value = Hsv::from_hex(&hex);
            
                hsv.set(hsv_value);
            } else {
                let Some(bx) = button.child().and_then(|child| child.downcast::<gtk4::Box>().ok()) else {
                    return;
                };

                let Some(label) = bx
                    .iter_children()
                    .find_map(|child| child.downcast::<gtk4::Label>().ok().filter(|lbl| lbl.widget_name() == "paste-from-clipboard-label"))
                else {
                    return;
                };

                label.set_text("No valid color in clipboard!");

                button_label_change_timeout.set(Duration::from_secs(2), move || label.set_text("Paste from Clipboard"));
            }
        }
    });

    color_tabs_stack.add_tab(Some("hex"), &hex_fields.widget);
    color_tabs_stack.add_tab(Some("int"), &int_fields.widget);
    color_tabs_stack.add_tab(Some("rgb"), &rgb_fields.widget);
    color_tabs_stack.add_tab(Some("hsv"), &hsv_fields.widget);
    color_tabs_stack.add_tab(Some("hsl"), &hsl_fields.widget);
    color_tabs_stack.add_tab(Some("cmyk"), &cmyk_fields.widget);
    color_tabs_stack.add_tab(Some("oklch"), &oklch_fields.widget);

    gtk4::glib::spawn_future_local(signal!(hsv, (hsv) {
        use fields::FieldUpdate::*;

        let rgba = hsv.as_rgba();
        let hsl = hsv.as_hsl();
        let cmyk = hsv.as_cmyk();
        let oklch = hsv.as_oklch();

        hex_fields.update(vec![Text(hsv.as_hex())]);
        int_fields.update(vec![Text(hsv.as_int().to_string())]);
        rgb_fields.update(vec![Float(rgba.red as f64), Float(rgba.green as f64), Float(rgba.blue as f64)]);
        hsv_fields.update(vec![Float(hsv.hue), Float(hsv.saturation), Float(hsv.value)]);
        hsl_fields.update(vec![Float(hsl.hue), Float(hsl.saturation), Float(hsl.lightness)]);
        cmyk_fields.update(vec![Float(cmyk.cyan as f64), Float(cmyk.magenta as f64), Float(cmyk.yellow as f64), Float(cmyk.black as f64)]);
        oklch_fields.update(vec![Float(oklch.lightness), Float(oklch.chroma), Float(oklch.hue)]);
    }));

    transform_tabs_stack.add_tab(Some("analogous"), &color_boxes::get_analogous_color_boxes(&hsv, 5, &color_tabs));
    transform_tabs_stack.add_tab(Some("triadic"), &color_boxes::get_analogous_color_boxes(&hsv, 3, &color_tabs));
    transform_tabs_stack.add_tab(Some("tetradic"), &color_boxes::get_analogous_color_boxes(&hsv, 4, &color_tabs));
    transform_tabs_stack.add_tab(Some("lighter_darker"), &color_boxes::get_lighter_darker_color_boxes(&hsv, 20, &color_tabs).grid);

    // Listen for IPC messages to update the HSV value
    ipc::listen_for_messages_local(move |message| {
        let mut split_whitespace_iterator = message.split_whitespace();
        if let Some(message) = split_whitespace_iterator.next()
            && message == "color_picker_set_hex"
            && let Some(hex) = split_whitespace_iterator.next()
        {
            let hsv_value = Hsv::from_hex(hex);

            hsv.set(hsv_value);
        }
    });

    widget
}