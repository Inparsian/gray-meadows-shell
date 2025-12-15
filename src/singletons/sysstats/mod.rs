pub mod sensors;
mod gpu {
    pub mod nvidia;
}

use std::time::Duration;
use std::sync::{Mutex, LazyLock};
use futures_signals::signal::Mutable;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::struct_wrappers::device::MemoryInfo;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind};

const REFRESH_INTERVAL: Duration = Duration::from_secs(1);

static SYS: LazyLock<Mutex<sysinfo::System>> = LazyLock::new(|| {
    Mutex::new(sysinfo::System::new_with_specifics(sysinfo::RefreshKind::nothing()
        .with_memory(MemoryRefreshKind::nothing().with_ram().with_swap())
        .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
    ))
});

pub static SYS_STATS: LazyLock<Mutex<SysStats>> = LazyLock::new(|| Mutex::new(SysStats::default()));

#[derive(Default)]
pub struct SysStats {
    // sysstats
    pub uptime: Mutable<u64>,
    pub used_memory: Mutable<u64>,
    pub total_memory: Mutable<u64>,
    pub free_memory: Mutable<u64>,
    pub used_swap: Mutable<u64>,
    pub total_swap: Mutable<u64>,
    pub free_swap: Mutable<u64>,
    pub global_cpu_usage: Mutable<f64>,

    // nvml
    pub gpu_utilization: Mutable<f64>,
    pub gpu_temperature: Mutable<f64>,
    pub gpu_free_memory: Mutable<u64>,
    pub gpu_used_memory: Mutable<u64>,
    pub gpu_total_memory: Mutable<u64>,
}

impl SysStats {
    pub fn refresh(&self) {
        let mut sys = SYS.lock().unwrap();
        sys.refresh_memory();
        sys.refresh_cpu_usage();

        self.uptime.set(sysinfo::System::uptime());
        self.used_memory.set(sys.used_memory());
        self.total_memory.set(sys.total_memory());
        self.free_memory.set(sys.free_memory());
        self.used_swap.set(sys.used_swap());
        self.total_swap.set(sys.total_swap());
        self.free_swap.set(sys.free_swap());
        self.global_cpu_usage.set(sys.global_cpu_usage() as f64);

        // Refresh GPU stats if NVML is initialized
        if let Ok(device) = gpu::nvidia::get_device_by_index(0) {
            match device.utilization_rates() {
                Ok(util) => self.gpu_utilization.set(util.gpu as f64),
                Err(err) => eprintln!("Failed to get GPU utilization: {:?}", err)
            }

            match device.temperature(TemperatureSensor::Gpu) {
                Ok(temp) => self.gpu_temperature.set(temp as f64),
                Err(err) => eprintln!("Failed to get GPU temperature: {:?}", err)
            }

            match device.memory_info() {
                Ok(MemoryInfo { total, used, free, .. }) => {
                    self.gpu_total_memory.set(total);
                    self.gpu_used_memory.set(used);
                    self.gpu_free_memory.set(free);
                },
                Err(err) => eprintln!("Failed to get GPU memory info: {:?}", err)
            }
        }
    }

    pub fn memory_usage_percentage(&self) -> f64 {
        if self.total_memory.get() == 0 {
            0.0
        } else {
            (self.used_memory.get() as f64 / self.total_memory.get() as f64) * 100.0
        }
    }

    pub fn swap_usage_percentage(&self) -> f64 {
        if self.total_swap.get() == 0 {
            0.0
        } else {
            (self.used_swap.get() as f64 / self.total_swap.get() as f64) * 100.0
        }
    }

    pub fn vram_usage_percentage(&self) -> f64 {
        if self.gpu_total_memory.get() == 0 {
            0.0
        } else {
            (self.gpu_used_memory.get() as f64 / self.gpu_total_memory.get() as f64) * 100.0
        }
    }
}

pub fn activate() {
    // TODO: Add support for other GPU vendors
    let _ = gpu::nvidia::init_nvml();
    sensors::init_sensors();
    
    SYS_STATS.lock().unwrap().uptime.set(sysinfo::System::uptime());

    std::thread::spawn(|| {
        loop {
            std::thread::sleep(REFRESH_INTERVAL);
            SYS_STATS.lock().unwrap().refresh();
        }
    });
}