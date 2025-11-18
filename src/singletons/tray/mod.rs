use std::sync::{Arc, Mutex, LazyLock, OnceLock};
use tokio::sync::broadcast;

use crate::singletons::tray::{bus::BusEvent, wrapper::{
    sn_item::StatusNotifierItem,
    sn_watcher::StatusNotifierWatcher
}};

pub mod proxy;
pub mod wrapper;
pub mod bus;
pub mod icon;
pub mod tray_menu;

static SENDER: LazyLock<broadcast::Sender<BusEvent>> = LazyLock::new(|| {
    broadcast::channel(100).0
});

pub static ITEMS: OnceLock<Arc<Mutex<Vec<StatusNotifierItem>>>> = OnceLock::new();

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

pub fn try_get_item(service: &str) -> Option<StatusNotifierItem> {
    ITEMS.get()?.try_lock().map_or(None, |items| items.iter()
        .find(|item| item.service == service)
        .cloned())
}