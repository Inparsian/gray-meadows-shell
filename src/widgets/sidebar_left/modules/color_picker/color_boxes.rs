use std::{rc::Rc, cell::RefCell};
use futures_signals::signal::{Mutable, SignalExt};
use gtk4::prelude::*;

use crate::{
    color::{model::Hsv, LighterDarkerResult},
    helpers::gesture,
    ipc,
    widgets::common::{dynamic_grid::DynamicGrid, tabs::Tabs}
};

#[derive(Debug, Clone)]
pub struct ColorBox {
    pub widget: gtk4::Overlay,
    pub css_provider: gtk4::CssProvider,
    pub hsv: Rc<RefCell<Hsv>>
}

pub fn get_color_box(hsv: Hsv, color_tabs: &Tabs) -> ColorBox {
    let hsv = Rc::new(RefCell::new(hsv));
    let current_tab = &color_tabs.current_tab;

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
                    let current_tab = current_tab.clone();
                    move |_| {
                        let hsv = hsv.borrow();
                        let text = match current_tab.get_cloned().as_deref() {
                            Some("int") => hsv.as_int().to_string(),
                            Some("rgb") => hsv.as_rgba().as_string(),
                            Some("hsv") => hsv.as_string(),
                            Some("hsl") => hsv.as_hsl().as_string(),
                            Some("cmyk") => hsv.as_cmyk().as_string(),
                            Some("oklch") => hsv.as_oklch().as_string(),
                            _ => hsv.as_hex()
                        };

                        // TODO: Do this without wl-copy?
                        std::thread::spawn(move || std::process::Command::new("wl-copy")
                            .arg(text)
                            .output()
                        );
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

pub fn get_lighter_darker_color_boxes(hsv: &Mutable<Hsv>, count: u32, color_tabs: &Tabs) -> DynamicGrid {
    let mut grid = DynamicGrid::new(4);
    let mut boxes: Vec<(ColorBox, gtk4::Label)> = Vec::new();

    for _ in 0..=count {
        let color_box = get_color_box(hsv.get(), color_tabs);
        let label = gtk4::Label::new(Some("0%"));
        let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        box_.append(&color_box.widget);
        box_.append(&label);

        grid.append(&box_);
        boxes.push((color_box, label));
    }

    let hsv_future = hsv.signal().for_each({
        let boxes = boxes.clone();
        let color = hsv.get();

        move |hsv| {
            let lighter_darker_colors = crate::color::get_lighter_darker_colors(hsv, count);
            let default_result = LighterDarkerResult {
                hsv: color,
                lightness: 0.0,
                is_original: false
            };
            
            for (i, (color_box, label)) in boxes.iter().enumerate() {
                let new_color = lighter_darker_colors.get(i).unwrap_or(&default_result);

                color_box.css_provider.load_from_data(&format!(
                    ".color-picker-transform-color {{ background-color: {}; }}",
                    new_color.hsv.as_hex()
                ));

                label.set_label(&format!("{:<4}", format!("{:.0}%", new_color.lightness)));

                if new_color.is_original {
                    label.set_css_classes(&["color-picker-transform-color-label", "original"]);
                } else {
                    label.set_css_classes(&["color-picker-transform-color-label"]);
                }

                let _ = color_box.hsv.try_borrow_mut().map(|mut c| *c = new_color.hsv);
            }

            async {}
        }
    });

    gtk4::glib::spawn_future_local(hsv_future);

    grid
}