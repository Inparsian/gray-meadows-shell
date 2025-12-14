use gdk4::cairo::{FontSlant, FontWeight};
use gtk4::prelude::*;
use futures_signals::signal::SignalExt as _;

use crate::singletons::sysstats;
use crate::widgets::common::radial_progress::{RadialProgress, RadialProgressOptions, RadialProgressTextOptions, RadialProgressTextSize};

pub fn radial_progress_options() -> RadialProgressOptions {
    let css_classes = vec!["bar-sysstats-extended-radial"];
    let radius = 40.0;
    let thickness = 4.0;
    let start_angle = -std::f64::consts::FRAC_PI_2;
    let clockwise = true;
    let top_text_options = RadialProgressTextOptions {
        font_size: RadialProgressTextSize::Big,
        weight: FontWeight::Bold,
        slant: FontSlant::Normal,
    };
    let bottom_text_options = RadialProgressTextOptions {
        font_size: RadialProgressTextSize::Normal,
        weight: FontWeight::Normal,
        slant: FontSlant::Normal,
    };

    RadialProgressOptions {
        css_classes,
        radius,
        thickness,
        start_angle,
        clockwise,
        top_text_options,
        bottom_text_options,
    }
}

pub fn extended() -> gtk4::Box {
    let cpu_usage_radial = RadialProgress::new(&radial_progress_options());
    let memory_usage_radial = RadialProgress::new(&radial_progress_options());

    view! {
        widget = gtk4::Box {
            set_css_classes: &["bar-sysstats-extended"],
            set_orientation: gtk4::Orientation::Horizontal,
            set_spacing: 4,

            append: &cpu_usage_radial.drawing_area,
            append: &memory_usage_radial.drawing_area,
        },
    }

    gtk4::glib::spawn_future_local(signal!(sysstats::SYS_STATS.lock().unwrap().global_cpu_usage, (cpu_usage) {
        cpu_usage_radial.set_progress(cpu_usage / 100.0);
        cpu_usage_radial.set_top_text(Some("CPU".to_owned()));
        cpu_usage_radial.set_bottom_text(Some(format!("{:.1}%", cpu_usage)));
    }));

    gtk4::glib::spawn_future_local(signal!(sysstats::SYS_STATS.lock().unwrap().used_memory, (used_memory) {
        let sys_stats = sysstats::SYS_STATS.lock().unwrap();
        let total_memory = sys_stats.total_memory.get();
        let memory_usage_percentage = (used_memory as f64 / total_memory as f64) * 100.0;

        memory_usage_radial.set_progress(memory_usage_percentage / 100.0);
        memory_usage_radial.set_top_text(Some("RAM".to_owned()));
        memory_usage_radial.set_bottom_text(Some(format!("{:.1}%", memory_usage_percentage)));
    }));

    widget
}