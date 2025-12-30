pub mod proxy;
pub mod wrapper;
pub mod bus;

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, OnceLock, RwLock};
use async_broadcast::Receiver;

use crate::broadcast::BroadcastChannel;
use self::bus::BusEvent;
use self::wrapper::Notification;

#[allow(dead_code)]
static CHANNEL: LazyLock<BroadcastChannel<BusEvent>> = LazyLock::new(|| BroadcastChannel::new(10));

pub static NOTIFICATIONS: OnceLock<Arc<RwLock<HashMap<u32, Notification>>>> = OnceLock::new();

pub fn activate() {
    let manager = wrapper::NotificationManager::new();
    let mut receiver = manager.subscribe();

    NOTIFICATIONS.set(manager.notifications())
        .expect("Failed to set the notifications for notification manager");

    tokio::spawn(async move {
        while let Ok(event) = receiver.recv().await {
            // DEBUG
            match &event {
                BusEvent::NotificationAdded(id, notification) => {
                    println!("Notification added ({}): {:#?}", id, notification);
                },

                BusEvent::NotificationUpdated(id, notification) => {
                    println!("Notification updated ({}): {:#?}", id, notification);
                },

                BusEvent::NotificationClosed(id) => {
                    println!("Notification removed: id={}", id);
                },

                _ => {},
            }

            // TODO: no channel subscribers yet
            //CHANNEL.send(event).await;
        }
    });

    std::thread::spawn(move || manager.serve());
}

#[allow(dead_code)]
pub fn subscribe() -> Receiver<bus::BusEvent> {
    CHANNEL.subscribe()
}