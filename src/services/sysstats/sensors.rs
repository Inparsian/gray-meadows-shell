use std::sync::LazyLock;
use futures_signals::signal::Mutable;

#[derive(Default)]
pub struct Sensors {
    pub cpu_temp: Mutable<f64>,
}

pub static SENSORS: LazyLock<Sensors> = LazyLock::new(Sensors::default);

pub fn init_sensors() {
    std::thread::spawn(|| {
        let sensors = sensors::Sensors::new();
        
        loop {
            // Try to find chips such as k10temp for AMD
            if let Some(k10temp) = sensors.into_iter().find(|c| c.get_name().is_ok_and(|n| n.starts_with("k10temp-pci")))
                && let Some(tctl) = k10temp.into_iter().find(|f| f.get_label().is_ok_and(|l| l == "Tctl"))
                && let Some(value) = tctl.into_iter().find_map(|sf| sf.get_value().ok())
            {
                SENSORS.cpu_temp.set(value);
            } else {
                warn!("No k10temp chip found or Tctl feature not found");
            }
            
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}