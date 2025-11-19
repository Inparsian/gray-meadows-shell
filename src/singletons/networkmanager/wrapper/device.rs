#![allow(dead_code)]
use crate::singletons::networkmanager::enums::*;

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
}

#[derive(Debug, Clone)]
pub struct NetworkManagerDevice {
    pub device_type: NetworkManagerDeviceType,
    pub state: (NetworkManagerDeviceState, NetworkManagerDeviceStateReason)
}

