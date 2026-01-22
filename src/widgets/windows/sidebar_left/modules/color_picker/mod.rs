mod hue_picker;
mod saturation_value_picker;
mod fields;
mod color_boxes;

use std::time::Duration;
use futures_signals::signal::{Mutable, SignalExt as _};
use gtk4::{Adjustment, prelude::*};

use crate::color::{parse_color_into_hex, int_to_hex};
use crate::color::models::{Rgba, Hsv, Hsl, Cmyk, Oklab, Oklch, ColorModel as _};
use crate::ipc;
use crate::singletons::clipboard;
use crate::utils::timeout::Timeout;
use crate::widgets::common::tabs::{TabSize, Tabs};
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

    let color_tabs = Tabs::new(TabSize::Normal, false, Some("color-picker-tabs-stack"));
    color_tabs.set_current_tab(Some("hsv"));

    let transform_tabs = Tabs::new(TabSize::Normal, false, Some("color-picker-tabs-stack"));
    transform_tabs.set_current_tab(Some("analogous"));

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
        let mut rgba = Rgba::from_model(hsv.get());
        rgba.red = value as u8;
        Hsv::from_model(rgba)
    });
    create_spin_field!(rgb_fields, 0, 1.0, Adjustment::new(0.0, 0.0, 255.0, 1.0, 1.0, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut rgba = Rgba::from_model(hsv.get());
        rgba.green = value as u8;
        Hsv::from_model(rgba)
    });
    create_spin_field!(rgb_fields, 0, 1.0, Adjustment::new(0.0, 0.0, 255.0, 1.0, 1.0, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut rgba = Rgba::from_model(hsv.get());
        rgba.blue = value as u8;
        Hsv::from_model(rgba)
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
        let mut hsl_value = Hsl::from_model(hsv.get());
        hsl_value.hue = value;
        Hsv::from_model(hsl_value)
    });
    create_spin_field!(hsl_fields, 2, 0.33, Adjustment::new(0.0, 0.0, 100.0, 0.33, 0.33, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut hsl_value = Hsl::from_model(hsv.get());
        hsl_value.saturation = value;
        Hsv::from_model(hsl_value)
    });
    create_spin_field!(hsl_fields, 2, 0.33, Adjustment::new(0.0, 0.0, 100.0, 0.33, 0.33, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut hsl_value = Hsl::from_model(hsv.get());
        hsl_value.lightness = value;
        Hsv::from_model(hsl_value)
    });

    let mut cmyk_fields = Fields::new();
    create_spin_field!(cmyk_fields, 0, 1.0, Adjustment::new(0.0, 0.0, 100.0, 1.0, 1.0, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut cmyk_value = Cmyk::from_model(hsv.get());
        cmyk_value.cyan = value as u8;
        Hsv::from_model(cmyk_value)
    });
    create_spin_field!(cmyk_fields, 0, 1.0, Adjustment::new(0.0, 0.0, 100.0, 1.0, 1.0, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut cmyk_value = Cmyk::from_model(hsv.get());
        cmyk_value.magenta = value as u8;
        Hsv::from_model(cmyk_value)
    });
    create_spin_field!(cmyk_fields, 0, 1.0, Adjustment::new(0.0, 0.0, 100.0, 1.0, 1.0, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut cmyk_value = Cmyk::from_model(hsv.get());
        cmyk_value.yellow = value as u8;
        Hsv::from_model(cmyk_value)
    });
    create_spin_field!(cmyk_fields, 0, 1.0, Adjustment::new(0.0, 0.0, 100.0, 1.0, 1.0, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut cmyk_value = Cmyk::from_model(hsv.get());
        cmyk_value.black = value as u8;
        Hsv::from_model(cmyk_value)
    });
    
    let mut oklab_fields = Fields::new();
    create_spin_field!(oklab_fields, 3, 0.033, Adjustment::new(0.0, 0.0, 100.0, 0.033, 0.033, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut oklab_value = Oklab::from_model(hsv.get());
        oklab_value.lightness = value;
        Hsv::from_model(oklab_value)
    });
    create_spin_field!(oklab_fields, 3, 0.0033, Adjustment::new(0.0, -0.4, 0.4, 0.0033, 0.0033, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut oklab_value = Oklab::from_model(hsv.get());
        oklab_value.a = value;
        Hsv::from_model(oklab_value)
    });
    create_spin_field!(oklab_fields, 3, 0.0033, Adjustment::new(0.0, -0.4, 0.4, 0.0033, 0.0033, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut oklab_value = Oklab::from_model(hsv.get());
        oklab_value.b = value;
        Hsv::from_model(oklab_value)
    });

    let mut oklch_fields = Fields::new();
    create_spin_field!(oklch_fields, 3, 0.033, Adjustment::new(0.0, 0.0, 100.0, 0.033, 0.033, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut oklch_value = Oklch::from_model(hsv.get());
        oklch_value.lightness = value;
        Hsv::from_model(oklch_value)
    });
    create_spin_field!(oklch_fields, 3, 0.033, Adjustment::new(0.0, 0.0, 100.0, 0.033, 0.033, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut oklch_value = Oklch::from_model(hsv.get());
        oklch_value.chroma = value;
        Hsv::from_model(oklch_value)
    });
    create_spin_field!(oklch_fields, 2, 0.33, Adjustment::new(0.0, 0.0, 360.0, 0.33, 0.33, 0.0), hsv, |hsv: &Mutable<Hsv>, value| {
        let mut oklch_value = Oklch::from_model(hsv.get());
        oklch_value.hue = value;
        Hsv::from_model(oklch_value)
    });

    view! {
        paste_from_clipboard_label = gtk4::Label {
            set_label: "Paste from Clipboard",
        },
        
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
                
                append: &paste_from_clipboard_label,
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
            append: &color_tabs.group()
                .spacing(12)
                .build(),
            append: &transform_tabs.group()
                .spacing(12)
                .build(),
        }
    }

    let button_label_change_timeout = Timeout::default();
    paste_from_clipboard_button.connect_clicked(clone!(
        #[strong] hsv,
        move |_| {
            glib::spawn_future_local(clone!(
                #[strong] hsv,
                #[weak] paste_from_clipboard_label,
                #[weak] button_label_change_timeout,
                async move {
                    let Some(clipboard_text) = clipboard::fetch_text_clipboard().await else {
                        return;
                    };
                    
                    if let Some(hex) = parse_color_into_hex(&clipboard_text) {
                        hsv.set(Hsv::from_hex(&hex));
                    } else {
                        paste_from_clipboard_label.set_text("No valid color in clipboard!");
        
                        button_label_change_timeout.set(
                            Duration::from_secs(2),
                            move || paste_from_clipboard_label.set_text("Paste from Clipboard")
                        );
                    }
                }
            ));
        }
    ));

    color_tabs.add_tab("HEX", "hex", None, &hex_fields.widget);
    color_tabs.add_tab("INT", "int", None, &int_fields.widget);
    color_tabs.add_tab("RGB", "rgb", None, &rgb_fields.widget);
    color_tabs.add_tab("HSV", "hsv", None, &hsv_fields.widget);
    color_tabs.add_tab("HSL", "hsl", None, &hsl_fields.widget);
    color_tabs.add_tab("CMYK", "cmyk", None, &cmyk_fields.widget);
    color_tabs.add_tab("OKLAB", "oklab", None, &oklab_fields.widget);
    color_tabs.add_tab("OKLCH", "oklch", None, &oklch_fields.widget);

    glib::spawn_future_local(signal!(hsv, (hsv) {
        use fields::FieldUpdate::*;

        let rgba = Rgba::from_model(hsv);
        let hsl = Hsl::from_model(hsv);
        let cmyk = Cmyk::from_model(hsv);
        let oklab = Oklab::from_model(hsv);
        let oklch = Oklch::from_model(hsv);

        hex_fields.update(vec![Text(hsv.into_hex())]);
        int_fields.update(vec![Text(hsv.into_int().to_string())]);
        rgb_fields.update(vec![Float(rgba.red as f64), Float(rgba.green as f64), Float(rgba.blue as f64)]);
        hsv_fields.update(vec![Float(hsv.hue), Float(hsv.saturation), Float(hsv.value)]);
        hsl_fields.update(vec![Float(hsl.hue), Float(hsl.saturation), Float(hsl.lightness)]);
        cmyk_fields.update(vec![Float(cmyk.cyan as f64), Float(cmyk.magenta as f64), Float(cmyk.yellow as f64), Float(cmyk.black as f64)]);
        oklab_fields.update(vec![Float(oklab.lightness), Float(oklab.a), Float(oklab.b)]);
        oklch_fields.update(vec![Float(oklch.lightness), Float(oklch.chroma), Float(oklch.hue)]);
    }));
    
    transform_tabs.add_tab("ANALOGOUS", "analogous", None, &color_boxes::get_analogous_color_boxes(&hsv, 5, &color_tabs));
    transform_tabs.add_tab("TRIADIC", "triadic", None, &color_boxes::get_analogous_color_boxes(&hsv, 3, &color_tabs));
    transform_tabs.add_tab("TETRADIC", "tetradic", None, &color_boxes::get_analogous_color_boxes(&hsv, 4, &color_tabs));
    transform_tabs.add_tab("LIGHTNESS", "lighter_darker", None, &color_boxes::get_lighter_darker_color_boxes(&hsv, 20, &color_tabs).grid);

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