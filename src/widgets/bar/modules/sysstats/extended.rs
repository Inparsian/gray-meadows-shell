use std::time::Duration;
use gdk4::glib::ControlFlow;
use gdk4::cairo::{FontSlant, FontWeight};
use gtk4::prelude::*;
use rand::Rng as _;

use crate::widgets::common::radial_progress::{RadialProgress, RadialProgressOptions, RadialProgressTextOptions, RadialProgressTextSize};

pub fn extended() -> gtk4::Box {
    let test_radial = RadialProgress::new(&RadialProgressOptions {
        css_classes: vec!["bar-sysstats-extended-radial"],
        radius: 40.0,
        thickness: 4.0,
        start_angle: -std::f64::consts::FRAC_PI_2,
        clockwise: true,
        top_text_options: RadialProgressTextOptions {
            font_size: RadialProgressTextSize::Big,
            weight: FontWeight::Bold,
            slant: FontSlant::Normal,
        },
        bottom_text_options: RadialProgressTextOptions {
            font_size: RadialProgressTextSize::Normal,
            weight: FontWeight::Normal,
            slant: FontSlant::Normal,
        },
    });

    view! {
        widget = gtk4::Box {
            set_css_classes: &["bar-sysstats-extended"],
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 0,

            append: &test_radial.drawing_area,
        },
    }

    gtk4::glib::timeout_add_local(Duration::from_secs(1), move || {
        let mut rng = rand::rng();
        let progress = rng.random();
        test_radial.set_progress(progress);
        test_radial.set_top_text(Some("CPU".to_owned()));
        test_radial.set_bottom_text(Some(format!("{:.0}%", progress * 100.0)));
        ControlFlow::Continue
    });

    widget
}