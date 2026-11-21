pub mod sensors;
mod gpu {
    pub mod nvidia;
}

use std::time::Duration;
use std::sync::{Mutex, LazyLock};
use futures_signals::signal::Mutable;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::struct_wrappers::device::MemoryInfo as NvmlMemoryInfo;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind};

use crate::utils::unit::bytes_to_gib;

const REFRESH_INTERVAL: Duration = Duration::from_secs(1);

static SYS: LazyLock<Mutex<sysinfo::System>> = LazyLock::new(|| {
    Mutex::new(sysinfo::System::new_with_specifics(sysinfo::RefreshKind::nothing()
        .with_memory(MemoryRefreshKind::nothing().with_ram().with_swap())
        .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
    ))
});

pub static SYS_STATS: LazyLock<SysStats> = LazyLock::new(SysStats::default);

#[derive(Default, Clone, Copy)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
}

impl MemoryInfo {
    pub fn usage_percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.used as f64 / self.total as f64) * 100.0
        }
    }
}

pub struct ZramDevice {
    pub path: String,
    pub size: u64,
    pub data: u64,
    pub compr: u64,
    pub total: u64,
}

impl ZramDevice {
    pub fn new(path: String) -> Self {
        let mut device = Self {
            path,
            size: 0,
            data: 0,
            compr: 0,
            total: 0,
        };
        device.refresh();
        device
    }

    pub fn refresh(&mut self) {
        let size_path = format!("{}/disksize", self.path);
        let stat_path = format!("{}/mm_stat", self.path); // in order: data compr total

        if let Ok(size_str) = std::fs::read_to_string(&size_path) {
            self.size = size_str.trim().parse::<u64>().unwrap_or(0);
        }

        if let Ok(stat_str) = std::fs::read_to_string(&stat_path) {
            let parts: Vec<&str> = stat_str.split_whitespace().collect();
            if parts.len() >= 3 {
                self.data = parts[0].parse::<u64>().unwrap_or(0);
                self.compr = parts[1].parse::<u64>().unwrap_or(0);
                self.total = parts[2].parse::<u64>().unwrap_or(0);
            }
        }
    }

    pub fn dev_path(&self) -> String {
        format!("/dev/{}", self.path.split('/').next_back().unwrap_or(""))
    }

    pub fn is_active(&self) -> bool {
        // if it's in /proc/swaps it is active
        if let Ok(swaps_str) = std::fs::read_to_string("/proc/swaps") {
            for line in swaps_str.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if !parts.is_empty() && parts[0] == self.dev_path() {
                    return true;
                }
            }
        }
        false
    }
}

pub struct ZramAggregate {
    pub size: u64,
    pub data: u64,
    pub compr: u64,
    pub used: u64,
}

#[derive(Default)]
pub struct ZramDevices {
    devices: Vec<ZramDevice>,
}

impl ZramDevices {
    pub fn add(&mut self, device: ZramDevice) {
        self.devices.push(device);
    }

    pub fn iter(&self) -> impl Iterator<Item = &ZramDevice> {
        self.devices.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut ZramDevice> {
        self.devices.iter_mut()
    }

    pub fn any_active(&self) -> bool {
        self.devices.iter().any(|device| device.is_active())
    }

    pub fn aggregate(&self) -> ZramAggregate {
        let mut total_size = 0;
        let mut total_data = 0;
        let mut total_compr = 0;
        let mut total_used = 0;

        for device in &self.devices {
            if !device.is_active() {
                continue;
            }
            total_size += device.size;
            total_data += device.data;
            total_compr += device.compr;
            total_used += device.total;
        }

        ZramAggregate { size: total_size, data: total_data, compr: total_compr, used: total_used, }
    }
}

#[derive(Default)]
pub struct SysStats {
    // sysstats
    pub uptime: Mutable<u64>,
    pub memory: Mutable<MemoryInfo>,
    pub swap: Mutable<MemoryInfo>,
    pub zram_devices: Mutable<ZramDevices>,
    pub global_cpu_usage: Mutable<f64>,

    // nvml
    pub gpu_utilization: Mutable<f64>,
    pub gpu_temperature: Mutable<f64>,
    pub gpu_memory: Mutable<MemoryInfo>,
}

impl SysStats {
    pub fn discover_zram(&self) {
        for entry in std::fs::read_dir("/sys/block").unwrap_or_else(|_| panic!("Failed to read /sys/block")) {
            let entry = entry.unwrap();
            let file_name = entry.file_name();
            if file_name.to_string_lossy().starts_with("zram") {
                let zram_device = ZramDevice::new(entry.path().to_string_lossy().to_string());
                let human_size = format!("{:.1} GiB", bytes_to_gib(zram_device.size));
                info!("Discovered zram device: {} (size: {}, active: {})", zram_device.path, human_size, if zram_device.is_active() { "yes" } else { "no" });
                self.zram_devices.lock_mut().add(zram_device);
            }
        }
    }

    pub fn refresh(&self) {
        let mut sys = SYS.lock().unwrap();
        sys.refresh_memory();
        sys.refresh_cpu_usage();

        self.uptime.set(sysinfo::System::uptime());
        self.memory.set(MemoryInfo { total: sys.total_memory(), used: sys.used_memory() });
        self.swap.set(MemoryInfo { total: sys.total_swap(), used: sys.used_swap() });
        self.global_cpu_usage.set(sys.global_cpu_usage() as f64);
        self.zram_devices.lock_mut().iter_mut().for_each(|device| device.refresh());

        // Refresh GPU stats if NVML is initialized
        if let Ok(device) = gpu::nvidia::get_device_by_index(0) {
            match device.utilization_rates() {
                Ok(util) => self.gpu_utilization.set(util.gpu as f64),
                Err(err) => warn!(?err, "Failed to get GPU utilization")
            }

            match device.temperature(TemperatureSensor::Gpu) {
                Ok(temp) => self.gpu_temperature.set(temp as f64),
                Err(err) => warn!(?err, "Failed to get GPU temperature")
            }

            match device.memory_info() {
                Ok(NvmlMemoryInfo { total, used, .. }) => self.gpu_memory.set(MemoryInfo {
                    total,
                    used,
                }),
                Err(err) => warn!(?err, "Failed to get GPU memory info")
            }
        }
    }
}

pub fn activate() {
    // TODO: Add support for other GPU vendors
    let _ = gpu::nvidia::init_nvml();
    sensors::init_sensors();
    SYS_STATS.discover_zram();

    std::thread::spawn(|| {
        loop {
            SYS_STATS.refresh();
            std::thread::sleep(REFRESH_INTERVAL);
        }
    });
}