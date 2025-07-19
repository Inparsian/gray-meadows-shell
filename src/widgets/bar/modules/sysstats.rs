use futures_signals::signal::{Mutable, SignalExt};
use gtk4::prelude::*;
use once_cell::sync::Lazy;

use crate::{
    helpers::{gesture, unit},
    singletons,
    widgets::bar::wrapper::BarModuleWrapper
};

const SWAP_SHOW_THRESHOLD: f64 = 5.0; // Show swap usage only if it's above this threshold, 
                                      // indicating that the system is under memory pressure.

static DETAILED: Lazy<Mutable<bool>> = Lazy::new(|| Mutable::new(false));                                      

fn format_percentage(percentage: f64) -> String {
    if percentage <= 100.0 {
        format!("{:02.0}%", percentage)
    } else {
        "</3".to_owned() // we show brokey heart cause rip in ripperoni
    }
}

fn get_ram_usage_label_text() -> String {
    let usage = singletons::sysstats::SYS_STATS.lock().unwrap().memory_usage_percentage();
    format_percentage(usage)
}

fn get_detailed_ram_usage_label_text() -> String {
    let sys_stats = singletons::sysstats::SYS_STATS.lock().unwrap();
    format!(
        "({:.1}/{:.1}GiB)",
        unit::bytes_to_gib(sys_stats.used_memory.get()),
        unit::bytes_to_gib(sys_stats.total_memory.get())
    )
}

fn get_swap_usage_label_text() -> String {
    let usage = singletons::sysstats::SYS_STATS.lock().unwrap().swap_usage_percentage();
    format_percentage(usage)
}

fn get_detailed_swap_usage_label_text() -> String {
    let sys_stats = singletons::sysstats::SYS_STATS.lock().unwrap();
    format!(
        "({:.1}/{:.1}GiB)",
        unit::bytes_to_gib(sys_stats.used_swap.get()),
        unit::bytes_to_gib(sys_stats.total_swap.get())
    )
}

fn get_cpu_usage_label_text() -> String {
    let usage = singletons::sysstats::SYS_STATS.lock().unwrap().global_cpu_usage.get();
    format_percentage(usage)
}

fn get_cpu_temperature_label_text() -> String {
    format!("({:.1}°C)", singletons::sysstats::sensors::SENSORS.cpu_temp.get())
}

fn get_gpu_usage_label_text() -> String {
    let sys_stats = singletons::sysstats::SYS_STATS.lock().unwrap();
    format_percentage(sys_stats.gpu_utilization.get())
}

fn get_gpu_temperature_label_text() -> String {
    let sys_stats = singletons::sysstats::SYS_STATS.lock().unwrap();
    format!(
        "({:.1}°C)",
        sys_stats.gpu_temperature.get()
    )
}

pub fn new() -> gtk4::Box {
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

        gtk4::glib::spawn_future_local(DETAILED.signal().for_each(move |detailed| {
            detailed_reveal.set_reveal_child(detailed);

            async {}
        }));

        box_
    };

    view! {
        detailed_toggle_gesture = gesture::on_primary_up(|_, _, _| DETAILED.set(!DETAILED.get())),
        
        ram_usage_label = gtk4::Label {
            set_label: &get_ram_usage_label_text(),
            set_halign: gtk4::Align::Start,
        },

        detailed_ram_usage_label = gtk4::Label {
            set_css_classes: &["bar-sysstats-detailed-label"],
            set_label: &get_detailed_ram_usage_label_text(),
            set_halign: gtk4::Align::Start,
        },

        swap_usage_label = gtk4::Label {
            set_label: &get_swap_usage_label_text(),
            set_halign: gtk4::Align::Start,
        },

        detailed_swap_usage_label = gtk4::Label {
            set_css_classes: &["bar-sysstats-detailed-label"],
            set_label: &get_detailed_swap_usage_label_text(),
            set_halign: gtk4::Align::Start,
        },

        cpu_usage_label = gtk4::Label {
            set_label: &get_cpu_usage_label_text(),
            set_halign: gtk4::Align::Start,
        },

        cpu_temperature_label = gtk4::Label {
            set_css_classes: &["bar-sysstats-detailed-label"],
            set_label: &get_cpu_temperature_label_text(),
            set_halign: gtk4::Align::Start,
        },

        gpu_usage_label = gtk4::Label {
            set_label: &get_gpu_usage_label_text(),
            set_halign: gtk4::Align::Start,
        },

        gpu_temperature_label = gtk4::Label {
            set_css_classes: &["bar-sysstats-detailed-label"],
            set_label: &get_gpu_temperature_label_text(),
            set_halign: gtk4::Align::Start,
        },

        // Rationale behind var binding: see SWAP_SHOW_THRESHOLD const.
        swap_usage_box = create_sysstats_item("", &swap_usage_label, &detailed_swap_usage_label),

        widget = gtk4::Box {
            set_css_classes: &["bar-widget", "bar-sysstats"],
            set_hexpand: false,

            create_sysstats_item("󰍛", &ram_usage_label, &detailed_ram_usage_label) {},
            append: &swap_usage_box,
            create_sysstats_item("󰻠", &cpu_usage_label, &cpu_temperature_label) {},
            create_sysstats_item("󰢮", &gpu_usage_label, &gpu_temperature_label) {}
        }
    }

    let ram_usage_future = singletons::sysstats::SYS_STATS.lock().unwrap().used_memory.signal().for_each(move |_| {
        ram_usage_label.set_label(&get_ram_usage_label_text());
        detailed_ram_usage_label.set_label(&get_detailed_ram_usage_label_text());

        async {}
    });

    let swap_usage_future = singletons::sysstats::SYS_STATS.lock().unwrap().used_swap.signal().for_each(move |_| {
        swap_usage_label.set_label(&get_swap_usage_label_text());
        detailed_swap_usage_label.set_label(&get_detailed_swap_usage_label_text());
        swap_usage_box.set_visible(singletons::sysstats::SYS_STATS.lock().unwrap().swap_usage_percentage() > SWAP_SHOW_THRESHOLD);

        async {}
    });

    let cpu_usage_future = singletons::sysstats::SYS_STATS.lock().unwrap().global_cpu_usage.signal().for_each(move |_| {
        cpu_usage_label.set_label(&get_cpu_usage_label_text());

        async {}
    });

    let cpu_temp_future = singletons::sysstats::sensors::SENSORS.cpu_temp.signal().for_each(move |_| {
        cpu_temperature_label.set_label(&get_cpu_temperature_label_text());

        async {}
    });

    let gpu_util_future = singletons::sysstats::SYS_STATS.lock().unwrap().gpu_utilization.signal().for_each(move |_| {
        gpu_usage_label.set_label(&get_gpu_usage_label_text());

        async {}
    });

    let gpu_temp_future = singletons::sysstats::SYS_STATS.lock().unwrap().gpu_temperature.signal().for_each(move |_| {
        gpu_temperature_label.set_label(&get_gpu_temperature_label_text());

        async {}
    });

    gtk4::glib::spawn_future_local(ram_usage_future);
    gtk4::glib::spawn_future_local(swap_usage_future);
    gtk4::glib::spawn_future_local(cpu_usage_future);
    gtk4::glib::spawn_future_local(cpu_temp_future);
    gtk4::glib::spawn_future_local(gpu_util_future);
    gtk4::glib::spawn_future_local(gpu_temp_future);

    BarModuleWrapper::new(&widget)
        .add_controller(detailed_toggle_gesture)
        .get_widget()
}