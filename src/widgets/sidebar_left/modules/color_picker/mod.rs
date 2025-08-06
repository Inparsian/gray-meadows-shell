mod hue_picker;
mod saturation_value_picker;
mod fields;

use std::{rc::Rc, cell::RefCell};
use futures_signals::signal::{Mutable, SignalExt};
use gtk4::prelude::*;

use crate::{
    color::model::{int_to_hex, Hsv},
    helpers::gesture,
    ipc,
    widgets::{common::tabs::{TabSize, Tabs, TabsStack}, sidebar_left::modules::color_picker::fields::Fields}
};

#[derive(Debug, Clone)]
pub struct ColorBox {
    pub widget: gtk4::Overlay,
    pub css_provider: gtk4::CssProvider,
    pub hsv: Rc<RefCell<Hsv>>
}

pub fn get_color_box(hsv: Hsv, _color_tabs: &Tabs) -> ColorBox {
    let hsv = Rc::new(RefCell::new(hsv));

    let (widget, css_provider) = {
        view! {
            color_box = gtk4::Box {
                set_css_classes: &["color-picker-transform-color"],
                set_hexpand: true,
            },

            color_copy_button = gtk4::Button {
                set_css_classes: &["color-picker-transform-copy-button"],
                connect_clicked: {
                    let hsv = hsv.clone();
                    move |_| {
                        println!("Copied color: {}", hsv.borrow().as_hex());
                    }
                },

                add_controller: gesture::on_secondary_up({
                    let hsv = hsv.clone();
                    move |_, _, _| {
                        let _ = ipc::client::send_message(&format!("color_picker_set_hex {}", hsv.borrow().as_hex()));
                    }
                }),

                gtk4::Label {
                    set_css_classes: &["material-icons"],
                    set_label: "content_copy",
                    set_hexpand: true
                }
            },

            color_copy_button_revealer = gtk4::Revealer {
                set_transition_type: gtk4::RevealerTransitionType::Crossfade,
                set_transition_duration: 200,
                set_reveal_child: false,
                add_controller: gesture::on_enter({
                    let revealer = color_copy_button_revealer.clone();
                    move |_, _| {
                        revealer.set_reveal_child(true);
                    }
                }),

                add_controller: gesture::on_leave({
                    let revealer = color_copy_button_revealer.clone();
                    move || {
                        revealer.set_reveal_child(false);
                    }
                }),

                set_child: Some(&color_copy_button)
            },

            color_overlay = gtk4::Overlay {
                set_child: Some(&color_box),
                add_overlay: &color_copy_button_revealer
            }
        };

        let css_provider = gtk4::CssProvider::new();
        color_box.style_context().add_provider(
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        (color_overlay, css_provider)
    };

    ColorBox {
        widget,
        css_provider,
        hsv
    }
}

pub fn get_analogous_color_boxes(hsv: &Mutable<Hsv>, count: u32, color_tabs: &Tabs) -> gtk4::Box {
    let box_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let mut boxes: Vec<ColorBox> = Vec::new();

    for _ in 0..count {
        let color_box = get_color_box(hsv.get(), color_tabs);

        box_container.append(&color_box.widget);
        boxes.push(color_box);
    }

    let hsv_future = hsv.signal().for_each({
        let boxes = boxes.clone();
        let color = hsv.get();

        move |hsv| {
            let analogous_colors = crate::color::get_analogous_colors(hsv, count);
            for (i, color_box) in boxes.iter().enumerate() {
                let new_color = analogous_colors.get(i).unwrap_or(&color);

                color_box.css_provider.load_from_data(&format!(
                    ".color-picker-transform-color {{ background-color: {}; }}",
                    new_color.as_hex()
                ));

                let _ = color_box.hsv.try_borrow_mut().map(|mut c| *c = *new_color);
            }

            async {}
        }
    });

    gtk4::glib::spawn_future_local(hsv_future);

    box_container
}

pub fn new() -> gtk4::Box {
    let hsv = Mutable::new(Hsv {
        hue: 0.0,
        saturation: 100.0,
        value: 100.0
    });

    let hue_picker = hue_picker::HuePicker::new(&hsv);
    let saturation_value_picker = saturation_value_picker::SaturationValuePicker::new(&hsv);

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

    let color_tabs_stack = TabsStack::new(&color_tabs, Some("color-picker-tabs-stack"));
    let transform_tabs_stack = TabsStack::new(&transform_tabs, Some("color-picker-tabs-stack"));

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

            append: &color_tabs.widget,
            append: &color_tabs_stack.widget,
            append: &transform_tabs.widget,
            append: &transform_tabs_stack.widget
        }
    }

    color_tabs_stack.add_tab(Some("hex"), &hex_fields.widget);
    color_tabs_stack.add_tab(Some("int"), &int_fields.widget);
    color_tabs_stack.add_tab(Some("rgb"), &rgb_fields.widget);
    color_tabs_stack.add_tab(Some("hsv"), &hsv_fields.widget);
    color_tabs_stack.add_tab(Some("hsl"), &hsl_fields.widget);
    color_tabs_stack.add_tab(Some("cmyk"), &cmyk_fields.widget);
    color_tabs_stack.add_tab(Some("oklch"), &oklch_fields.widget);

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

    transform_tabs_stack.add_tab(Some("analogous"), &get_analogous_color_boxes(&hsv, 5, &color_tabs));
    transform_tabs_stack.add_tab(Some("triadic"), &get_analogous_color_boxes(&hsv, 3, &color_tabs));
    transform_tabs_stack.add_tab(Some("tetradic"), &get_analogous_color_boxes(&hsv, 4, &color_tabs));

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