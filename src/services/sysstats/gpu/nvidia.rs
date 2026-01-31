use std::sync::OnceLock;
use nvml_wrapper::{Nvml, Device, error::NvmlError};

pub static NVML: OnceLock<Nvml> = OnceLock::new();

pub fn init_nvml() -> Result<(), NvmlError> {
    let _ = NVML.set(Nvml::init()?);

    Ok(())
}

pub fn get_device_by_index<'a>(index: u32) -> Result<Device<'a>, NvmlError> {
    let nvml = NVML.get().ok_or(NvmlError::Uninitialized)?;

    nvml.device_by_index(index)
}