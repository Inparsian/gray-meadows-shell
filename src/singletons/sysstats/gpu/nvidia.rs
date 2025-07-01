use nvml_wrapper::{Nvml, error::NvmlError};
use once_cell::sync::OnceCell;

pub static NVML: OnceCell<Nvml> = OnceCell::new();

pub fn init_nvml() -> Result<(), NvmlError> {
    let _ = NVML.set(Nvml::init()?);

    Ok(())
}

pub fn get_device_by_index<'a>(index: u32) -> Result<nvml_wrapper::Device<'a>, NvmlError> {
    let nvml = NVML.get().ok_or(NvmlError::Uninitialized)?;

    nvml.device_by_index(index)
}