use std::{rc::Rc, cell::RefCell};
use futures_signals::signal::{Mutable, SignalExt as _};
use gtk4::prelude::*;

use crate::ipc;
use crate::color::LighterDarkerResult;
use crate::color::models::{Rgba, Hsv, Hsl, Cmyk, Oklab, Oklch, ColorModel as _};
use crate::singletons::clipboard;
use crate::utils::gesture;
use crate::widgets::common::{dynamic_grid::DynamicGrid, tabs::Tabs};

#[derive(Debug, Clone)]
pub struct ColorBox {
    pub widget: gtk4::Overlay,
    pub css_provider: gtk4::CssProvider,
    pub hsv: Rc<RefCell<Hsv>>
}

pub fn get_color_box(hsv: Hsv, color_tabs: &Tabs) -> ColorBox {
    let hsv = Rc::new(RefCell::new(hsv));
    let current_tab = &color_tabs.current_tab;

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
                        Some("int") => hsv.into_int().to_string(),
                        Some("rgb") => Rgba::from_model(*hsv).into_string(),
                        Some("hsv") => hsv.into_string(),
                        Some("hsl") => Hsl::from_model(*hsv).into_string(),
                        Some("cmyk") => Cmyk::from_model(*hsv).into_string(),
                        Some("oklab") => Oklab::from_model(*hsv).into_string(),
                        Some("oklch") => Oklch::from_model(*hsv).into_string(),
                        _ => hsv.into_hex()
                    };

                    std::thread::spawn(move || clipboard::copy_text(&text));
                }
            },

            add_controller: gesture::on_secondary_up({
                let hsv = hsv.clone();
                move |_, _, _| {
                    let _ = ipc::client::send_message(&format!("color_picker_set_hex {}", hsv.borrow().into_hex()));
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
                move |_, _| revealer.set_reveal_child(true)
            }),

            add_controller: gesture::on_leave({
                let revealer = color_copy_button_revealer.clone();
                move || revealer.set_reveal_child(false)
            }),

            set_child: Some(&color_copy_button)
        },

        widget = gtk4::Overlay {
            set_child: Some(&color_box),
            add_overlay: &color_copy_button_revealer
        }
    };

    let css_provider = gtk4::CssProvider::new();
    color_box.style_context().add_provider(
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    ColorBox {
        widget,
        css_provider,
        hsv
    }
}

pub fn get_analogous_color_boxes(hsv: &Mutable<Hsv>, count: u32, color_tabs: &Tabs) -> gtk4::Box {
    let box_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    box_container.set_valign(gtk4::Align::Start);
    
    let mut boxes: Vec<ColorBox> = Vec::new();

    for _ in 0..count {
        let color_box = get_color_box(hsv.get(), color_tabs);

        box_container.append(&color_box.widget);
        boxes.push(color_box);
    }

    gtk4::glib::spawn_future_local({
        let boxes = boxes.clone();
        let color = hsv.get();
        signal!(hsv, (hsv) {
            let analogous_colors = crate::color::get_analogous_colors(hsv, count);
            for (i, color_box) in boxes.iter().enumerate() {
                let new_color = analogous_colors.get(i).unwrap_or(&color);

                color_box.css_provider.load_from_data(&format!(
                    ".color-picker-transform-color {{ background-color: {}; }}",
                    new_color.into_hex()
                ));

                let _ = color_box.hsv.try_borrow_mut().map(|mut c| *c = *new_color);
            }
        })
    });

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

    gtk4::glib::spawn_future_local({
        let boxes = boxes.clone();
        let color = hsv.get();
        signal!(hsv, (hsv) {
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
                    new_color.hsv.into_hex()
                ));

                label.set_label(&format!("{:<4}", format!("{:.0}%", new_color.lightness)));

                if new_color.is_original {
                    label.set_css_classes(&["color-picker-transform-color-label", "original"]);
                } else {
                    label.set_css_classes(&["color-picker-transform-color-label"]);
                }

                let _ = color_box.hsv.try_borrow_mut().map(|mut c| *c = new_color.hsv);
            }
        })
    });

    grid
}