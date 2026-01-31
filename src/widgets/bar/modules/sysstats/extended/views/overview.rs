use gtk4::prelude::*;
use futures_signals::signal::SignalExt as _;

use crate::services::sysstats::SYS_STATS;
use crate::services::sysstats::sensors::SENSORS;
use crate::utils::unit::bytes_to_gib;
use super::super::CompactStatRow;
use super::super::super::SWAP_SHOW_THRESHOLD;

pub fn overview() -> gtk4::Box {
    let cpu_stat_row = CompactStatRow::new("CPU", true);
    let mem_stat_row = CompactStatRow::new("RAM", true);
    let swap_stat_row = CompactStatRow::new("SWAP", true);
    let gpu_stat_row = CompactStatRow::new("GPU", true);
    let vram_stat_row = CompactStatRow::new("VRAM", true);

    view! {
        widget = gtk4::Box {
            set_orientation: gtk4::Orientation::Vertical,
            set_spacing: 8,

            append: &cpu_stat_row.container,
            append: &mem_stat_row.container,
            append: &swap_stat_row.container,
            append: &gpu_stat_row.container,
            append: &vram_stat_row.container,
        },
    }

    glib::spawn_future_local({
        let cpu_stat_row = cpu_stat_row.clone();
        signal!(SYS_STATS.global_cpu_usage, (cpu_usage) {
            cpu_stat_row.set_value(&format!("{:.1}%", cpu_usage));
        })
    });

    glib::spawn_future_local(signal!(SENSORS.cpu_temp, (cpu_temp) {
        cpu_stat_row.set_secondary_value(&format!("{:.1}°C", cpu_temp));
    }));

    glib::spawn_future_local(signal!(SYS_STATS.memory, (memory) {
        mem_stat_row.set_value(&format!("{:.1} / {:.1} GiB", bytes_to_gib(memory.used), bytes_to_gib(memory.total)));
        mem_stat_row.set_secondary_value(&format!("{:.1}%", memory.usage_percentage()));
    }));

    glib::spawn_future_local(signal!(SYS_STATS.swap, (swap) {
        swap_stat_row.set_value(&format!("{:.1} / {:.1} GiB", bytes_to_gib(swap.used), bytes_to_gib(swap.total)));
        swap_stat_row.set_secondary_value(&format!("{:.1}%", swap.usage_percentage()));

        if swap.usage_percentage() <= SWAP_SHOW_THRESHOLD {
            swap_stat_row.container.add_css_class("irrelevant");
        } else {
            swap_stat_row.container.remove_css_class("irrelevant");
        }
    }));

    glib::spawn_future_local({
        let gpu_stat_row = gpu_stat_row.clone();
        signal!(SYS_STATS.gpu_utilization, (gpu_utilization) {
            gpu_stat_row.set_value(&format!("{:.1}%", gpu_utilization));
        })
    });
    
    glib::spawn_future_local(signal!(SYS_STATS.gpu_temperature, (gpu_temperature) {
        gpu_stat_row.set_secondary_value(&format!("{:.1}°C", gpu_temperature));
    }));

    glib::spawn_future_local(signal!(SYS_STATS.gpu_memory, (gpu_memory) {
        vram_stat_row.set_value(&format!("{:.1} / {:.1} GiB", bytes_to_gib(gpu_memory.used), bytes_to_gib(gpu_memory.total)));
        vram_stat_row.set_secondary_value(&format!("{:.1}%", gpu_memory.usage_percentage()));
    }));

    widget
}