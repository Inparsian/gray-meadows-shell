#![allow(unused_imports)]
#![allow(dead_code)]
use std::{sync::{Arc, Mutex}, time::Duration};
use dbus::{blocking, channel::MatchingReceiver, message::MatchRule, MessageType};
use dbus_crossroads::{Crossroads, IfaceToken};

use crate::singletons::networkmanager::{bus::{self, BusEvent}, proxy::{self, manager::OrgFreedesktopNetworkManager}};

/// https://www.networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.html
#[derive(Debug, Clone)]
pub struct NetworkManager {
    sender: tokio::sync::broadcast::Sender<BusEvent>,
    // We don't implement the rest of the fields for simplicity's sake.
}