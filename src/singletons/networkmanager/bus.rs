#![allow(dead_code)]
pub const NM_BUS_NAME: &str = "org.freedesktop.NetworkManager";
pub const NM_MANAGER_PATH: &str = "/org/freedesktop/NetworkManager";
pub const NM_DEVICES_PATH: &str = "/org/freedesktop/NetworkManager/Devices";
pub const NM_ACCESSPOINT_PATH: &str = "/org/freedesktop/NetworkManager/AccessPoint";

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum BusEvent {
    /// A NetworkManager device was added.
    DeviceAdded(String),

    /// A NetworkManager device was removed.
    DeviceRemoved(String),
}