use futures_signals::signal::SignalExt;
use gtk4::prelude::*;

use crate::singletons;

const SWAP_SHOW_THRESHOLD: f64 = 5.0; // Show swap usage only if it's above this threshold, 
                                      // indicating that the system is under memory pressure.

fn format_percentage(percentage: f64) -> String {
    if percentage <= 100.0 {
        format!("{:02.0}%", percentage)
    } else {
        "</3".to_string() // we show brokey heart cause rip in ripperoni
    }
}

fn get_ram_usage_label_text() -> String {
    let usage = singletons::sysstats::SYS_STATS.lock().unwrap().memory_usage_percentage();
    format_percentage(usage)
}

fn get_swap_usage_label_text() -> String {
    let usage = singletons::sysstats::SYS_STATS.lock().unwrap().swap_usage_percentage();
    format_percentage(usage)
}

fn get_cpu_usage_label_text() -> String {
    let usage = singletons::sysstats::SYS_STATS.lock().unwrap().global_cpu_usage.get();
    format_percentage(usage)
}

pub fn new() -> gtk4::Box {
    fn create_sysstats_item(icon: &str, label: &gtk4::Label) -> gtk4::Box {
        let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        box_.set_css_classes(&["bar-sysstats-item"]);
        box_.append(&gtk4::Label::new(Some(icon)));
        box_.append(label);

        box_
    }

    relm4_macros::view! {
        ram_usage_label = gtk4::Label {
            set_label: &get_ram_usage_label_text(),
            set_halign: gtk4::Align::Start,
        },

        swap_usage_label = gtk4::Label {
            set_label: &get_swap_usage_label_text(),
            set_halign: gtk4::Align::Start,
        },

        cpu_usage_label = gtk4::Label {
            set_label: &get_cpu_usage_label_text(),
            set_halign: gtk4::Align::Start,
        },

        // Rationale behind var binding: see SWAP_SHOW_THRESHOLD const.
        swap_usage_box = create_sysstats_item("", &swap_usage_label),

        widget = gtk4::Box {
            set_css_classes: &["bar-widget", "bar-sysstats"],
            set_hexpand: false,

            create_sysstats_item("󰍛", &ram_usage_label) {},
            append: &swap_usage_box,
            create_sysstats_item("󰻠", &cpu_usage_label) {},
        }
    }

    let ram_usage_future = singletons::sysstats::SYS_STATS.lock().unwrap().used_memory.signal().for_each(move |_| {
        ram_usage_label.set_label(&get_ram_usage_label_text());

        async {}
    });

    let swap_usage_future = singletons::sysstats::SYS_STATS.lock().unwrap().used_swap.signal().for_each(move |_| {
        swap_usage_label.set_label(&get_swap_usage_label_text());
        swap_usage_box.set_visible(singletons::sysstats::SYS_STATS.lock().unwrap().swap_usage_percentage() > SWAP_SHOW_THRESHOLD);

        async {}
    });

    let cpu_usage_future = singletons::sysstats::SYS_STATS.lock().unwrap().global_cpu_usage.signal().for_each(move |_| {
        cpu_usage_label.set_label(&get_cpu_usage_label_text());

        async {}
    });

    gtk4::glib::MainContext::default().spawn_local(ram_usage_future);
    gtk4::glib::MainContext::default().spawn_local(swap_usage_future);
    gtk4::glib::MainContext::default().spawn_local(cpu_usage_future);

    widget
}