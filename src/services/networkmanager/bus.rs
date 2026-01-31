#![allow(dead_code)]
pub const NM_BUS_NAME: &str = "org.freedesktop.NetworkManager";
pub const NM_MANAGER_PATH: &str = "/org/freedesktop/NetworkManager";
pub const NM_DEVICES_PATH: &str = "/org/freedesktop/NetworkManager/Devices";
pub const NM_ACCESSPOINT_PATH: &str = "/org/freedesktop/NetworkManager/AccessPoint";

use super::{enums::*, wrapper::{access_point::NetworkManagerAccessPoint, device::NetworkManagerDevice}};

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum BusEvent {
    /// The state of NetworkManager has changed.
    StateChanged(NetworkManagerState),

    /// A NetworkManager device was added.
    DeviceAdded(NetworkManagerDevice),

    /// A NetworkManager device was removed.
    DeviceRemoved(NetworkManagerDevice),

    /// The state of a NetworkManager device has changed.
    DeviceStateChanged(String, NetworkManagerDeviceState, NetworkManagerDeviceStateReason),

    /// A wireless device has discovered an access point.
    AccessPointAdded(NetworkManagerDevice, NetworkManagerAccessPoint),

    /// A wireless device has lost an access point.
    AccessPointRemoved(NetworkManagerDevice, NetworkManagerAccessPoint),
}