use std::sync::LazyLock;
use futures_signals::signal::{Mutable, SignalExt as _};
use gtk::prelude::*;

use crate::services::sysstats::{MemoryInfo, SYS_STATS};
use crate::services::sysstats::sensors::SENSORS;
use crate::utils::unit::bytes_to_gib;
use super::SWAP_SHOW_THRESHOLD;

pub static DETAILED: LazyLock<Mutable<bool>> = LazyLock::new(|| Mutable::new(false));

fn format_percentage(percentage: f64) -> String {
    if percentage <= 100.0 {
        format!("{:02.0}%", percentage)
    } else {
        "</3".to_owned() // we show brokey heart cause rip in ripperoni
    }
}

fn get_detailed_memory_usage_label_text(memory: &MemoryInfo) -> String {
    format!(
        "({:.1}/{:.1}GiB)",
        bytes_to_gib(memory.used),
        bytes_to_gib(memory.total)
    )
}

fn get_temperature_label_text(cpu_temp: f64) -> String {
    format!("({:.1}°C)", cpu_temp)
}

pub fn minimal() -> gtk::Box {
    let create_sysstats_item = |icon: &str, label: &gtk::Label, detail_label: &gtk::Label| -> gtk::Box {
        let box_ = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        let detailed_reveal = gtk::Revealer::new();
        detailed_reveal.set_transition_type(gtk::RevealerTransitionType::SlideRight);
        detailed_reveal.set_transition_duration(200);
        detailed_reveal.set_reveal_child(DETAILED.get());
        detailed_reveal.set_child(Some(detail_label));

        let icon_label = gtk::Label::new(Some(icon));
        icon_label.set_css_classes(&["bar-sysstats-icon"]);

        box_.set_css_classes(&["bar-sysstats-item"]);
        box_.append(&icon_label);
        box_.append(label);
        box_.append(&detailed_reveal);

        glib::spawn_future_local(signal!(DETAILED, (detailed) {
            detailed_reveal.set_reveal_child(detailed);
        }));

        box_
    };

    view! {        
        ram_usage_label = gtk::Label {
            set_halign: gtk::Align::Start,
        },

        detailed_ram_usage_label = gtk::Label {
            set_css_classes: &["bar-sysstats-detailed-label"],
            set_halign: gtk::Align::Start,
        },

        swap_usage_label = gtk::Label {
            set_halign: gtk::Align::Start,
        },

        detailed_swap_usage_label = gtk::Label {
            set_css_classes: &["bar-sysstats-detailed-label"],
            set_halign: gtk::Align::Start,
        },

        cpu_usage_label = gtk::Label {
            set_halign: gtk::Align::Start,
        },

        cpu_temperature_label = gtk::Label {
            set_css_classes: &["bar-sysstats-detailed-label"],
            set_halign: gtk::Align::Start,
        },

        gpu_usage_label = gtk::Label {
            set_halign: gtk::Align::Start,
        },

        gpu_temperature_label = gtk::Label {
            set_css_classes: &["bar-sysstats-detailed-label"],
            set_halign: gtk::Align::Start,
        },

        swap_usage_box = create_sysstats_item("", &swap_usage_label, &detailed_swap_usage_label),

        widget = gtk::Box {
            set_hexpand: false,

            create_sysstats_item("󰍛", &ram_usage_label, &detailed_ram_usage_label) {},
            append: &swap_usage_box,
            create_sysstats_item("󰻠", &cpu_usage_label, &cpu_temperature_label) {},
            create_sysstats_item("󰢮", &gpu_usage_label, &gpu_temperature_label) {}
        }
    }

    glib::spawn_future_local(signal!(SYS_STATS.memory, (memory) {
        ram_usage_label.set_label(&format_percentage(memory.usage_percentage()));
        detailed_ram_usage_label.set_label(&get_detailed_memory_usage_label_text(&memory));
    }));

    glib::spawn_future_local(signal!(SYS_STATS.swap, (swap) {
        swap_usage_label.set_label(&format_percentage(swap.usage_percentage()));
        detailed_swap_usage_label.set_label(&get_detailed_memory_usage_label_text(&swap));
        swap_usage_box.set_visible(swap.usage_percentage() > SWAP_SHOW_THRESHOLD);
    }));

    glib::spawn_future_local(signal!(SYS_STATS.global_cpu_usage, (global_cpu_usage) {
        cpu_usage_label.set_label(&format_percentage(global_cpu_usage));
    }));

    glib::spawn_future_local(signal!(SENSORS.cpu_temp, (cpu_temp) {
        cpu_temperature_label.set_label(&get_temperature_label_text(cpu_temp));
    }));

    glib::spawn_future_local(signal!(SYS_STATS.gpu_utilization, (gpu_utilization) {
        gpu_usage_label.set_label(&format_percentage(gpu_utilization));
    }));
    
    glib::spawn_future_local(signal!(SYS_STATS.gpu_temperature, (gpu_temperature) {
        gpu_temperature_label.set_label(&get_temperature_label_text(gpu_temperature));
    }));

    widget
}