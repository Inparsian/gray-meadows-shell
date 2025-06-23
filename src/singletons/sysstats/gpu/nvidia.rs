use std::sync::Mutex;
use nvml_wrapper::Nvml;
use once_cell::sync::Lazy;

static NVML: Lazy<Nvml> = Lazy::new(|| Nvml::init().unwrap());
pub static NVML_DEVICE: Lazy<Mutex<Option<nvml_wrapper::Device>>> = Lazy::new(|| Mutex::new(None));

pub fn init_nvml() -> Result<(), nvml_wrapper::error::NvmlError> {
    // Get the first available GPU device - we'll get it once and use it throughout the application
    let device = NVML.device_by_index(0)?;
    *NVML_DEVICE.lock().unwrap() = Some(device);

    Ok(())
}