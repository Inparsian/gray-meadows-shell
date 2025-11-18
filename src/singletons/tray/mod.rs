use std::sync::{Arc, LazyLock, OnceLock, RwLock};
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

pub static ITEMS: OnceLock<Arc<RwLock<Vec<StatusNotifierItem>>>> = OnceLock::new();

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

pub fn try_read_item(service: &str) -> Option<StatusNotifierItem> {
    let items = ITEMS.get()?.try_read().ok()?;
    items.iter()
        .find(|item| item.service == service)
        .cloned()
}