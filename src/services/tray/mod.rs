pub mod proxy;
pub mod wrapper;
pub mod bus;
pub mod icon;
pub mod tray_menu;

use std::sync::{Arc, LazyLock, OnceLock, RwLock};
use async_broadcast::Receiver;

use crate::utils::broadcast::BroadcastChannel;
use self::bus::BusEvent;
use self::wrapper::sn_item::StatusNotifierItem;
use self::wrapper::sn_watcher::StatusNotifierWatcher;

static CHANNEL: LazyLock<BroadcastChannel<BusEvent>> = LazyLock::new(|| BroadcastChannel::new(10));

pub static ITEMS: OnceLock<Arc<RwLock<Vec<StatusNotifierItem>>>> = OnceLock::new();

pub fn activate() {
    let watcher = StatusNotifierWatcher::default();
    let mut receiver = watcher.subscribe();

    ITEMS.set(watcher.items())
        .expect("Failed to set the items for tray watcher");

    tokio::spawn(async move {
        while let Ok(event) = receiver.recv().await {
            CHANNEL.send(event).await;
        }
    });

    std::thread::spawn(move || watcher.serve());
}

pub fn subscribe() -> Receiver<bus::BusEvent> {
    CHANNEL.subscribe()
}

pub fn try_read_item(service: &str) -> Option<StatusNotifierItem> {
    let items = ITEMS.get()?.try_read().ok()?;
    items.iter()
        .find(|item| item.service == service)
        .cloned()
}