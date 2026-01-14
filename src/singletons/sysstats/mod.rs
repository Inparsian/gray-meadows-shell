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

#[derive(Default)]
pub struct SysStats {
    // sysstats
    pub uptime: Mutable<u64>,
    pub memory: Mutable<MemoryInfo>,
    pub swap: Mutable<MemoryInfo>,
    pub global_cpu_usage: Mutable<f64>,

    // nvml
    pub gpu_utilization: Mutable<f64>,
    pub gpu_temperature: Mutable<f64>,
    pub gpu_memory: Mutable<MemoryInfo>,
}

impl SysStats {
    pub fn refresh(&self) {
        let mut sys = SYS.lock().unwrap();
        sys.refresh_memory();
        sys.refresh_cpu_usage();

        self.uptime.set(sysinfo::System::uptime());
        self.memory.set(MemoryInfo {
            total: sys.total_memory(),
            used: sys.used_memory(),
        });
        self.swap.set(MemoryInfo {
            total: sys.total_swap(),
            used: sys.used_swap(),
        });
        self.global_cpu_usage.set(sys.global_cpu_usage() as f64);

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

    std::thread::spawn(|| {
        loop {
            SYS_STATS.refresh();
            std::thread::sleep(REFRESH_INTERVAL);
        }
    });
}