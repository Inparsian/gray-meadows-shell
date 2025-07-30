mod hue_picker;
mod saturation_value_picker;

use futures_signals::signal::{Mutable, SignalExt};
use gtk4::prelude::*;

use crate::{color::model::Hsv, widgets::common::tabs::{TabSize, Tabs, TabsStack}};

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
    tabs.add_tab("RGBA", "rgba".to_owned(), None);
    tabs.add_tab("HSV", "hsv".to_owned(), None);
    tabs.add_tab("HSL", "hsl".to_owned(), None);
    tabs.add_tab("CMYK", "cmyk".to_owned(), None);
    tabs.add_tab("OKLCH", "oklch".to_owned(), None);

    let tabs_stack = TabsStack::new(&tabs, None);

    view! {
        test_hex_label = gtk4::Label {},
        test_int_label = gtk4::Label {},
        test_rgba_label = gtk4::Label {},
        test_hsv_label = gtk4::Label {},
        test_hsl_label = gtk4::Label {},
        test_cmyk_label = gtk4::Label {},
        test_oklch_label = gtk4::Label {},

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

    tabs_stack.add_tab(Some("hex"), &test_hex_label);
    tabs_stack.add_tab(Some("int"), &test_int_label);
    tabs_stack.add_tab(Some("rgba"), &test_rgba_label);
    tabs_stack.add_tab(Some("hsv"), &test_hsv_label);
    tabs_stack.add_tab(Some("hsl"), &test_hsl_label);
    tabs_stack.add_tab(Some("cmyk"), &test_cmyk_label);
    tabs_stack.add_tab(Some("oklch"), &test_oklch_label);

    let hsv_future = hsv.signal().for_each(move |hsv| {
        let hex = hsv.as_hex();
        let int = hsv.as_int();
        let rgba = hsv.as_rgba();
        let hsl = hsv.as_hsl();
        let cmyk = hsv.as_cmyk();
        let oklch = hsv.as_oklch();

        test_hex_label.set_text(&format!("Hex: {}", hex));
        test_int_label.set_text(&format!("Int: {}", int));
        test_rgba_label.set_text(&format!("RGBA: {:.2}, {:.2}, {:.2}, {:.2}", rgba.red, rgba.green, rgba.blue, rgba.alpha));
        test_hsv_label.set_text(&format!("HSV: {:.2}, {:.2}, {:.2}", hsv.hue, hsv.saturation, hsv.value));
        test_hsl_label.set_text(&format!("HSL: {:.2}, {:.2}, {:.2}", hsl.hue, hsl.saturation, hsl.lightness));
        test_cmyk_label.set_text(&format!("CMYK: {:.2}, {:.2}, {:.2}, {:.2}", cmyk.cyan, cmyk.magenta, cmyk.yellow, cmyk.black));
        test_oklch_label.set_text(&format!("OKLCH: {:.4}, {:.4}, {:.2}", oklch.lightness, oklch.chroma, oklch.hue));

        async {}
    });

    gtk4::glib::spawn_future_local(hsv_future);

    widget
}