use gtk4::prelude::*;
use futures_signals::signal::SignalExt as _;

use crate::singletons::sysstats::SYS_STATS;
use crate::singletons::sysstats::sensors::SENSORS;
use crate::unit::bytes_to_gib;
use super::SWAP_SHOW_THRESHOLD;

#[derive(Clone)]
pub struct CompactStatRow {
    pub container: gtk4::Box,
    pub value: gtk4::Label,
    pub secondary_value: gtk4::Label, // e.g. cpu temp next to cpu usage
}

impl CompactStatRow {
    pub fn new(label_text: &str, with_secondary: bool) -> Self {
        let container = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        container.set_css_classes(&["bar-sysstats-compact-stat-row"]);
        let label = gtk4::Label::new(None);
        label.set_text(label_text);
        label.set_hexpand(true);
        label.set_halign(gtk4::Align::Start);
        label.set_css_classes(&["bar-sysstats-compact-stat-row-label"]);
        let value = gtk4::Label::new(None);
        value.set_css_classes(&["bar-sysstats-compact-stat-row-value"]);
        value.set_xalign(1.0);
        let secondary_value = gtk4::Label::new(None);
        secondary_value.set_css_classes(&["bar-sysstats-compact-stat-row-secondary-value"]);
        secondary_value.set_xalign(1.0);

        container.append(&label);
        container.append(&value);

        if with_secondary {
            container.append(&secondary_value);
        }

        Self {
            container,
            value,
            secondary_value,
        }
    }

    pub fn set_value(&self, text: &str) {
        self.value.set_text(text);
    }

    pub fn set_secondary_value(&self, text: &str) {
        self.secondary_value.set_text(format!("({})", text).as_str());
    }
}

pub fn extended() -> gtk4::Box {
    let cpu_stat_row = CompactStatRow::new("CPU", true);
    let mem_stat_row = CompactStatRow::new("RAM", true);
    let swap_stat_row = CompactStatRow::new("SWAP", true);
    let gpu_stat_row = CompactStatRow::new("GPU", true);
    let vram_stat_row = CompactStatRow::new("VRAM", true);

    view! {
        widget = gtk4::Box {
            set_css_classes: &["bar-sysstats-extended"],
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 8,

            append: &cpu_stat_row.container,
            append: &mem_stat_row.container,
            append: &swap_stat_row.container,
            append: &gpu_stat_row.container,
            append: &vram_stat_row.container,
        },
    }

    gtk4::glib::spawn_future_local({
        let cpu_stat_row = cpu_stat_row.clone();
        signal!(SYS_STATS.lock().unwrap().global_cpu_usage, (cpu_usage) {
            cpu_stat_row.set_value(&format!("{:.1}%", cpu_usage));
        })
    });

    gtk4::glib::spawn_future_local(signal!(SENSORS.cpu_temp, (cpu_temp) {
        cpu_stat_row.set_secondary_value(&format!("{:.1}°C", cpu_temp));
    }));

    gtk4::glib::spawn_future_local(signal!(SYS_STATS.lock().unwrap().memory, (memory) {
        mem_stat_row.set_value(&format!("{:.1} / {:.1} GB", bytes_to_gib(memory.used), bytes_to_gib(memory.total)));
        mem_stat_row.set_secondary_value(&format!("{:.1}%", memory.usage_percentage()));
    }));

    gtk4::glib::spawn_future_local(signal!(SYS_STATS.lock().unwrap().swap, (swap) {
        swap_stat_row.set_value(&format!("{:.1} / {:.1} GB", bytes_to_gib(swap.used), bytes_to_gib(swap.total)));
        swap_stat_row.set_secondary_value(&format!("{:.1}%", swap.usage_percentage()));

        if swap.usage_percentage() <= SWAP_SHOW_THRESHOLD {
            swap_stat_row.container.add_css_class("irrelevant");
        } else {
            swap_stat_row.container.remove_css_class("irrelevant");
        }
    }));

    gtk4::glib::spawn_future_local({
        let gpu_stat_row = gpu_stat_row.clone();
        signal!(SYS_STATS.lock().unwrap().gpu_utilization, (gpu_utilization) {
            gpu_stat_row.set_value(&format!("{:.1}%", gpu_utilization));
        })
    });
    
    gtk4::glib::spawn_future_local(signal!(SYS_STATS.lock().unwrap().gpu_temperature, (gpu_temperature) {
        gpu_stat_row.set_secondary_value(&format!("{:.1}°C", gpu_temperature));
    }));

    gtk4::glib::spawn_future_local(signal!(SYS_STATS.lock().unwrap().gpu_memory, (gpu_memory) {
        vram_stat_row.set_value(&format!("{:.1} / {:.1} GB", bytes_to_gib(gpu_memory.used), bytes_to_gib(gpu_memory.total)));
        vram_stat_row.set_secondary_value(&format!("{:.1}%", gpu_memory.usage_percentage()));
    }));

    widget
}