use std::sync::LazyLock;
use futures_signals::signal::Mutable;
use lm_sensors::Initializer;

// The LM sensors wrapper I'm using is FUNNI - it doesn't implement Send + Sync,
// so it can't be used in multi-threaded contexts. To work around this, I'm using a custom
// struct and creating a LazyLock that holds it instead of a LMSensors lib instance.
#[derive(Default)]
pub struct Sensors {
    pub cpu_temp: Mutable<f64>,
}

pub static SENSORS: LazyLock<Sensors> = LazyLock::new(Sensors::default);

pub fn init_sensors() {
    // Spawn in a separate thread to run LM sensors on for the lifetime of the app
    std::thread::spawn(|| {
        match Initializer::default().initialize() {
            Ok(lm_sensors) => {
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(1));

                    // Try to find chips such as k10temp for AMD
                    let k10temp = lm_sensors.chip_iter(None)
                        .find(|chip| chip.name().unwrap_or_default().starts_with("k10temp-pci"));

                    if let Some(k10temp) = k10temp {
                        let tctl_feature = k10temp.feature_iter()
                            .find(|feature| feature.to_string() == "Tctl");

                        if let Some(tctl_feature) = tctl_feature {
                            for sub_feature in tctl_feature.sub_feature_iter() {
                                if let Ok(value) = sub_feature.raw_value() {
                                    SENSORS.cpu_temp.set(value);
                                    break;
                                }
                            }
                        } else {
                            warn!("Tctl feature not found in k10temp chip");
                        }
                    } else {
                        warn!("No k10temp chip found");
                    }
                }
            },

            Err(err) => {
                error!(?err, "Failed to initialize LM sensors");
            }
        }
    });
}