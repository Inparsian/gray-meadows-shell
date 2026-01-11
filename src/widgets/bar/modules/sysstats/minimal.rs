use std::sync::LazyLock;
use futures_signals::signal::{Mutable, SignalExt as _};
use gtk4::prelude::*;

use crate::singletons::sysstats::{MemoryInfo, SYS_STATS};
use crate::singletons::sysstats::sensors::SENSORS;
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

pub fn minimal() -> gtk4::Box {
    let create_sysstats_item = |icon: &str, label: &gtk4::Label, detail_label: &gtk4::Label| -> gtk4::Box {
        let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

        let detailed_reveal = gtk4::Revealer::new();
        detailed_reveal.set_transition_type(gtk4::RevealerTransitionType::SlideRight);
        detailed_reveal.set_transition_duration(200);
        detailed_reveal.set_reveal_child(DETAILED.get());
        detailed_reveal.set_child(Some(detail_label));

        let icon_label = gtk4::Label::new(Some(icon));
        icon_label.set_css_classes(&["bar-sysstats-icon"]);

        box_.set_css_classes(&["bar-sysstats-item"]);
        box_.append(&icon_label);
        box_.append(label);
        box_.append(&detailed_reveal);

        gtk4::glib::spawn_future_local(signal!(DETAILED, (detailed) {
            detailed_reveal.set_reveal_child(detailed);
        }));

        box_
    };

    view! {        
        ram_usage_label = gtk4::Label {
            set_halign: gtk4::Align::Start,
        },

        detailed_ram_usage_label = gtk4::Label {
            set_css_classes: &["bar-sysstats-detailed-label"],
            set_halign: gtk4::Align::Start,
        },

        swap_usage_label = gtk4::Label {
            set_halign: gtk4::Align::Start,
        },

        detailed_swap_usage_label = gtk4::Label {
            set_css_classes: &["bar-sysstats-detailed-label"],
            set_halign: gtk4::Align::Start,
        },

        cpu_usage_label = gtk4::Label {
            set_halign: gtk4::Align::Start,
        },

        cpu_temperature_label = gtk4::Label {
            set_css_classes: &["bar-sysstats-detailed-label"],
            set_halign: gtk4::Align::Start,
        },

        gpu_usage_label = gtk4::Label {
            set_halign: gtk4::Align::Start,
        },

        gpu_temperature_label = gtk4::Label {
            set_css_classes: &["bar-sysstats-detailed-label"],
            set_halign: gtk4::Align::Start,
        },

        swap_usage_box = create_sysstats_item("", &swap_usage_label, &detailed_swap_usage_label),

        widget = gtk4::Box {
            set_hexpand: false,

            create_sysstats_item("󰍛", &ram_usage_label, &detailed_ram_usage_label) {},
            append: &swap_usage_box,
            create_sysstats_item("󰻠", &cpu_usage_label, &cpu_temperature_label) {},
            create_sysstats_item("󰢮", &gpu_usage_label, &gpu_temperature_label) {}
        }
    }

    gtk4::glib::spawn_future_local(signal!(SYS_STATS.memory, (memory) {
        ram_usage_label.set_label(&format_percentage(memory.usage_percentage()));
        detailed_ram_usage_label.set_label(&get_detailed_memory_usage_label_text(&memory));
    }));

    gtk4::glib::spawn_future_local(signal!(SYS_STATS.swap, (swap) {
        swap_usage_label.set_label(&format_percentage(swap.usage_percentage()));
        detailed_swap_usage_label.set_label(&get_detailed_memory_usage_label_text(&swap));
        swap_usage_box.set_visible(swap.usage_percentage() > SWAP_SHOW_THRESHOLD);
    }));

    gtk4::glib::spawn_future_local(signal!(SYS_STATS.global_cpu_usage, (global_cpu_usage) {
        cpu_usage_label.set_label(&format_percentage(global_cpu_usage));
    }));

    gtk4::glib::spawn_future_local(signal!(SENSORS.cpu_temp, (cpu_temp) {
        cpu_temperature_label.set_label(&get_temperature_label_text(cpu_temp));
    }));

    gtk4::glib::spawn_future_local(signal!(SYS_STATS.gpu_utilization, (gpu_utilization) {
        gpu_usage_label.set_label(&format_percentage(gpu_utilization));
    }));
    
    gtk4::glib::spawn_future_local(signal!(SYS_STATS.gpu_temperature, (gpu_temperature) {
        gpu_temperature_label.set_label(&get_temperature_label_text(gpu_temperature));
    }));

    widget
}