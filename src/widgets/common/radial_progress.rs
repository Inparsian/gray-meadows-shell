#![allow(dead_code)]
use std::{rc::Rc, cell::RefCell};
use gdk4::cairo::{FontSlant, FontWeight};
use gtk4::prelude::*;

use crate::scss;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum RadialProgressTextSize {
    Small,
    Normal,
    Big,
}

impl RadialProgressTextSize {
    pub fn to_scss_user_variable(self) -> &'static str {
        match self {
            RadialProgressTextSize::Small => "font-family-small",
            RadialProgressTextSize::Normal => "font-family-normal",
            RadialProgressTextSize::Big => "font-family-big",
        }
    }

    pub fn to_font_size(self) -> f64 {
        // TODO: Make these configurable via SCSS variables
        match self {
            RadialProgressTextSize::Small => 11.0,
            RadialProgressTextSize::Normal => 13.0,
            RadialProgressTextSize::Big => 14.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RadialProgressTextOptions {
    pub font_size: RadialProgressTextSize,
    pub weight: FontWeight,
    pub slant: FontSlant,
}

#[derive(Debug, Clone)]
pub struct RadialProgressOptions {
    pub css_classes: Vec<&'static str>,
    pub radius: f64,
    pub thickness: f64,
    pub start_angle: f64,
    pub clockwise: bool,
    pub top_text_options: RadialProgressTextOptions,
    pub bottom_text_options: RadialProgressTextOptions,
}

pub struct RadialProgress {
    pub drawing_area: gtk4::DrawingArea,
    pub progress: Rc<RefCell<f64>>,
    pub top_text: Rc<RefCell<Option<String>>>,
    pub bottom_text: Rc<RefCell<Option<String>>>,
}

impl RadialProgress {
    pub fn new(options: &RadialProgressOptions) -> Self {
        let progress = Rc::new(RefCell::new(0.0));
        let top_text: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
        let bottom_text: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

        let drawing_area = gtk4::DrawingArea::new();
        drawing_area.add_css_class("radial-progress");
        drawing_area.set_width_request((options.radius * 2.0) as i32);
        drawing_area.set_height_request((options.radius * 2.0) as i32);
        for class in &options.css_classes {
            drawing_area.add_css_class(class);
        }

        drawing_area.set_draw_func({
            let options = options.clone();
            let progress = progress.clone();
            let top_text = top_text.clone();
            let bottom_text = bottom_text.clone();
            move |_, cr, width, height| {
                let progress = *progress.borrow();
                let center_x = width as f64 / 2.0;
                let center_y = height as f64 / 2.0;
                let radius = options.radius - options.thickness / 2.0;

                cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
                cr.set_line_width(options.thickness);
                cr.arc(center_x, center_y, radius, 0.0, 2.0 * std::f64::consts::PI);
                cr.stroke().unwrap();

                let (r, g, b) = scss::get_color("foreground-color-primary").map_or(
                    (226.0, 226.0, 226.0), 
                    |wave_color| (wave_color.red as f64, wave_color.green as f64, wave_color.blue as f64)
                );

                let arc = |alpha: f64, angle1: f64, angle2: f64| {
                    cr.set_source_rgba(r / 255.0, g / 255.0, b / 255.0, alpha);
                    cr.set_line_width(options.thickness);
                    cr.arc(
                        center_x,
                        center_y,
                        radius,
                        angle1,
                        angle2,
                    );
                    cr.stroke().unwrap();
                };

                let end_angle = options.start_angle
                    + if options.clockwise {
                        progress * 2.0 * std::f64::consts::PI
                    } else {
                        -progress * 2.0 * std::f64::consts::PI
                    };

                arc(0.3, 0.0, 2.0 * std::f64::consts::PI);
                arc(1.0, options.start_angle, end_angle);

                // Draw text if available
                if let Some(text) = &*top_text.borrow() {
                    let font_face = scss::get_string(options.top_text_options.font_size.to_scss_user_variable())
                        .unwrap_or_else(|| "monospace".to_owned());

                    cr.select_font_face(&font_face, options.top_text_options.slant, options.top_text_options.weight);
                    cr.set_font_size(options.top_text_options.font_size.to_font_size());
                    let extents = cr.text_extents(text).unwrap();
                    cr.set_source_rgba(r / 255.0, g / 255.0, b / 255.0, 1.0);
                    cr.move_to(
                        center_x - extents.width() / 2.0 - extents.x_bearing(),
                        center_y - 5.0,
                    );
                    cr.show_text(text).unwrap();
                }

                if let Some(text) = &*bottom_text.borrow() {
                    let font_face = scss::get_string(options.bottom_text_options.font_size.to_scss_user_variable())
                        .unwrap_or_else(|| "monospace".to_owned());

                    cr.select_font_face(&font_face, options.bottom_text_options.slant, options.bottom_text_options.weight);
                    cr.set_font_size(options.bottom_text_options.font_size.to_font_size());
                    let extents = cr.text_extents(text).unwrap();
                    cr.set_source_rgba(r / 255.0, g / 255.0, b / 255.0, 1.0);
                    cr.move_to(
                        center_x - extents.width() / 2.0 - extents.x_bearing(),
                        center_y + 15.0,
                    );
                    cr.show_text(text).unwrap();
                }
            }
        });

        Self {
            drawing_area,
            progress,
            top_text,
            bottom_text,
        }
    }

    pub fn set_progress(&self, value: f64) {
        *self.progress.borrow_mut() = value.clamp(0.0, 1.0);
        self.drawing_area.queue_draw();
    }

    pub fn set_top_text(&self, text: Option<String>) {
        *self.top_text.borrow_mut() = text;
        self.drawing_area.queue_draw();
    }

    pub fn set_bottom_text(&self, text: Option<String>) {
        *self.bottom_text.borrow_mut() = text;
        self.drawing_area.queue_draw();
    }
}