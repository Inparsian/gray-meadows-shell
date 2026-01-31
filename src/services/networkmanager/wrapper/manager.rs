#![allow(dead_code)]
use std::sync::{Arc, RwLock};

use crate::utils::broadcast::BroadcastChannel;
use super::device::NetworkManagerDevice;
use super::super::bus::BusEvent;

/// https://www.networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.html
#[derive(Debug, Clone)]
pub struct NetworkManager {
    channel: BroadcastChannel<BusEvent>,
    devices: Arc<RwLock<Vec<NetworkManagerDevice>>>,
    networking_enabled: bool,
    wireless_enabled: bool,
    // We don't implement the rest of the fields for simplicity's sake.
}