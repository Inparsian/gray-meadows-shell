use std::collections::{HashMap, HashSet};

use futures_signals::signal::SignalExt as _;
use gtk::prelude::*;

use crate::services::sysstats::SYS_STATS;
use crate::utils::unit::bytes_to_gib;
use super::super::CompactStatRow;
use super::super::super::SWAP_SHOW_THRESHOLD;

struct ZramDeviceRows {
    container: gtk::Box,
    disk_size: CompactStatRow,
    data: CompactStatRow,
    compr: CompactStatRow,
    total: CompactStatRow,
}

impl ZramDeviceRows {
    fn new(device_name: &str) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let title = gtk::Label::new(Some(device_name));
        title.set_halign(gtk::Align::Start);
        title.set_css_classes(&["bar-sysstats-compact-stat-row-label"]);

        let disk_size = CompactStatRow::new("DISKSIZE", false);
        let data = CompactStatRow::new("DATA", true);
        let compr = CompactStatRow::new("COMPR", true);
        let total = CompactStatRow::new("TOTAL", true);

        container.append(&title);
        container.append(&disk_size.container);
        container.append(&data.container);
        container.append(&compr.container);
        container.append(&total.container);

        Self {
            container,
            disk_size,
            data,
            compr,
            total,
        }
    }

    fn update(&self, size: u64, data: u64, compr: u64, total: u64) {
        let data_percentage = if size == 0 {
            0.0
        } else {
            data as f64 / size as f64 * 100.0
        };
        let compression_ratio = if compr == 0 {
            "-".to_owned()
        } else {
            format!("{:.2}x", data as f64 / compr as f64)
        };

        self.disk_size
            .set_value(&format!("{:.1} GiB", bytes_to_gib(size)));
        self.data
            .set_value(&format!("{:.1} GiB", bytes_to_gib(data)));
        self.data
            .set_secondary_value(&format!("{:.1}% of DISKSIZE", data_percentage));
        self.compr
            .set_value(&format!("{:.1} GiB", bytes_to_gib(compr)));
        self.compr
            .set_secondary_value(&format!("{} compression", compression_ratio));
        self.total
            .set_value(&format!("{:.1} GiB", bytes_to_gib(total)));
        self.total.set_secondary_value("RAM allocated");
    }
}

pub fn swap() -> gtk::Box {
    let swap_stat_row = CompactStatRow::new("SWAP", true);
    let zram_devices_box = gtk::Box::new(gtk::Orientation::Vertical, 8);

    view! {
        widget = gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,

            append: &swap_stat_row.container,
            append: &zram_devices_box,
        },
    }

    glib::spawn_future_local(signal!(SYS_STATS.swap, (swap) {
        swap_stat_row.set_value(&format!(
            "{:.1} / {:.1} GiB",
            bytes_to_gib(swap.used),
            bytes_to_gib(swap.total)
        ));
        swap_stat_row.set_secondary_value(&format!("{:.1}%", swap.usage_percentage()));
        if swap.usage_percentage() <= SWAP_SHOW_THRESHOLD {
            swap_stat_row.container.add_css_class("irrelevant");
        } else {
            swap_stat_row.container.remove_css_class("irrelevant");
        }
    }));

    glib::spawn_future_local({
        let mut device_rows: HashMap<String, ZramDeviceRows> = HashMap::new();
        SYS_STATS.zram_devices.signal_ref(|devices| {
            devices
                .iter()
                .filter(|device| device.is_active())
                .map(|device| {
                    (
                        device.dev_path(),
                        device.size,
                        device.data,
                        device.compr,
                        device.total,
                    )
                })
                .collect::<Vec<_>>()
        }).for_each(move |active_devices| {
            let active_paths = active_devices
                .iter()
                .map(|(path, _, _, _, _)| path.clone())
                .collect::<HashSet<_>>();

            device_rows.retain(|path, rows| {
                if active_paths.contains(path) {
                    true
                } else {
                    zram_devices_box.remove(&rows.container);
                    false
                }
            });

            for (path, size, data, compr, total) in active_devices {
                let rows = device_rows.entry(path.clone()).or_insert_with(|| {
                    let rows = ZramDeviceRows::new(&path);
                    zram_devices_box.append(&rows.container);
                    rows
                });
                rows.update(size, data, compr, total);
            }

            async {}
        })
    });

    widget
}
