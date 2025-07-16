use once_cell::sync::{Lazy, OnceCell};
use tokio::sync::broadcast;
use std::sync::{Arc, Mutex};

use crate::singletons::tray::{bus::BusEvent, wrapper::{
    sn_item::StatusNotifierItem,
    sn_watcher::StatusNotifierWatcher
}};

pub mod proxy;
pub mod wrapper;
pub mod bus;
pub mod icon;
pub mod tray_menu;

static SENDER: Lazy<broadcast::Sender<BusEvent>> = Lazy::new(|| {
    broadcast::channel(1).0
});

pub static ITEMS: OnceCell<Arc<Mutex<Vec<StatusNotifierItem>>>> = OnceCell::new();

pub fn activate() {
    let watcher = StatusNotifierWatcher::new();
    let mut receiver = watcher.subscribe();

    ITEMS.set(watcher.items())
        .expect("Failed to set the items for tray watcher");

    tokio::spawn(async move {
        while let Ok(event) = receiver.recv().await {
            let _ = SENDER.send(event);
        }
    });

    std::thread::spawn(move || watcher.serve());
}

pub fn subscribe() -> tokio::sync::broadcast::Receiver<bus::BusEvent> {
    SENDER.subscribe()
}

pub fn get_item(service: &str) -> Option<StatusNotifierItem> {
    ITEMS.get()?.lock().unwrap().iter()
        .find(|item| item.service == service)
        .cloned()
}

pub fn try_get_item(service: &str) -> Option<StatusNotifierItem> {
    if let Ok(items) = ITEMS.get()?.try_lock() {
        items.iter()
            .find(|item| item.service == service)
            .cloned()
    } else {
        None
    }
}