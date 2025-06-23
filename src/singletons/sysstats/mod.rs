pub mod sensors;
mod gpu {
    pub mod nvidia;
}

use futures_signals::signal::Mutable;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use once_cell::sync::Lazy;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind};
use std::{time::Duration, sync::Mutex};

use crate::singletons;

const REFRESH_INTERVAL: Duration = Duration::from_secs(1);

static SYS: Lazy<Mutex<sysinfo::System>> = Lazy::new(|| {
    Mutex::new(sysinfo::System::new_with_specifics(sysinfo::RefreshKind::nothing()
        .with_memory(MemoryRefreshKind::nothing().with_ram().with_swap())
        .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
    ))
});

pub static SYS_STATS: Lazy<Mutex<SysStats>> = Lazy::new(|| {
    Mutex::new(SysStats {
        used_memory: Mutable::new(0),
        total_memory: Mutable::new(0),
        free_memory: Mutable::new(0),
        used_swap: Mutable::new(0),
        total_swap: Mutable::new(0),
        free_swap: Mutable::new(0),
        global_cpu_usage: Mutable::new(0.0),
        gpu_utilization: Mutable::new(0.0),
        gpu_temperature: Mutable::new(0.0)
    })
});

pub struct SysStats {
    // sysstats
    pub used_memory: Mutable<u64>,
    pub total_memory: Mutable<u64>,
    pub free_memory: Mutable<u64>,
    pub used_swap: Mutable<u64>,
    pub total_swap: Mutable<u64>,
    pub free_swap: Mutable<u64>,
    pub global_cpu_usage: Mutable<f64>,

    // nvml
    pub gpu_utilization: Mutable<f64>,
    pub gpu_temperature: Mutable<f64>
}

impl SysStats {
    pub fn refresh(&self) {
        let mut sys = SYS.lock().unwrap();
        sys.refresh_memory();
        sys.refresh_cpu_usage();

        self.used_memory.set(sys.used_memory());
        self.total_memory.set(sys.total_memory());
        self.free_memory.set(sys.free_memory());
        self.used_swap.set(sys.used_swap());
        self.total_swap.set(sys.total_swap());
        self.free_swap.set(sys.free_swap());
        self.global_cpu_usage.set(sys.global_cpu_usage() as f64);

        // Refresh GPU stats if NVML is initialized
        if let Some(device) = singletons::sysstats::gpu::nvidia::NVML_DEVICE.lock().unwrap().as_ref() {
            let util = device.utilization_rates();
            if let Ok(util) = util {
                self.gpu_utilization.set(util.gpu as f64);
            } else {
                eprintln!("Failed to get GPU utilization: {:?}", util);
            }

            let temp = device.temperature(TemperatureSensor::Gpu);
            if let Ok(temp) = temp {
                self.gpu_temperature.set(temp as f64);
            } else {
                eprintln!("Failed to get GPU temperature: {:?}", temp);
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
}

pub fn activate() {
    // TODO: Add support for other GPU vendors
    let _ = singletons::sysstats::gpu::nvidia::init_nvml();
    singletons::sysstats::sensors::init_sensors();

    std::thread::spawn(|| {
        loop {
            std::thread::sleep(REFRESH_INTERVAL);
            SYS_STATS.lock().unwrap().refresh();
        }
    });
}