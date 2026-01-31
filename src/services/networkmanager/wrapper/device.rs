#![allow(dead_code)]
use std::sync::{Arc, RwLock};

use super::super::{enums::*, wrapper::access_point::NetworkManagerAccessPoint};

#[derive(Debug, Clone)]
pub enum NetworkManagerDeviceType {
    Wired(NetworkManagerDeviceWired),
    Wireless(NetworkManagerDeviceWireless),
    None,
}

#[derive(Debug, Clone)]
pub struct NetworkManagerDeviceWired {
    pub speed: u64, // in Mbps
}

#[derive(Debug, Clone)]
pub struct NetworkManagerDeviceWireless {
    pub access_points: Arc<RwLock<Vec<NetworkManagerAccessPoint>>>,
    pub active_access_point: String, // by bssid
}

#[derive(Debug, Clone)]
pub struct NetworkManagerDevice {
    pub device_type: NetworkManagerDeviceType,
    pub hw_address: String,
    pub perm_hw_address: String,
    pub state: (NetworkManagerDeviceState, NetworkManagerDeviceStateReason),
    pub flags: u32, // a bitmap, see enums
}

impl NetworkManagerDevice {
    pub fn flags_has_bit(&self, bit: NetworkManagerDeviceInterfaceFlags) -> bool {
        (self.flags & (bit as u32)) != 0
    }
}

