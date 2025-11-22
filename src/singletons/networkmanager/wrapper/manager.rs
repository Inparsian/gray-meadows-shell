#![allow(dead_code)]
use std::sync::{Arc, RwLock};

use crate::singletons::networkmanager::{bus::BusEvent, wrapper::device::NetworkManagerDevice};

/// https://www.networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.html
#[derive(Debug, Clone)]
pub struct NetworkManager {
    sender: tokio::sync::broadcast::Sender<BusEvent>,
    devices: Arc<RwLock<Vec<NetworkManagerDevice>>>,
    networking_enabled: bool,
    wireless_enabled: bool,
    // We don't implement the rest of the fields for simplicity's sake.
}