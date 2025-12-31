pub mod proxy;
pub mod wrapper;
pub mod bus;

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, OnceLock, RwLock};
use async_broadcast::Receiver;

use crate::broadcast::BroadcastChannel;
use self::bus::BusEvent;
use self::wrapper::{Notification, NotificationCloseReason, NotificationManager};

static CHANNEL: LazyLock<BroadcastChannel<BusEvent>> = LazyLock::new(|| BroadcastChannel::new(10));

pub static NOTIFICATIONS: OnceLock<Arc<RwLock<HashMap<u32, Notification>>>> = OnceLock::new();

pub fn activate() {
    let manager = NotificationManager::default();
    let mut receiver = manager.subscribe();

    NOTIFICATIONS.set(manager.notifications())
        .expect("Failed to set the notifications for notification manager");

    tokio::spawn(async move {
        while let Ok(event) = receiver.recv().await {
            CHANNEL.send(event).await;
        }
    });

    std::thread::spawn(move || manager.serve());
}

pub fn subscribe() -> Receiver<bus::BusEvent> {
    CHANNEL.subscribe()
}

pub fn close_notification_by_id(
    id: u32,
    reason: NotificationCloseReason
) -> Result<(), dbus::MethodErr> {
    let notifications = NOTIFICATIONS.get()
        .expect("Notifications singleton is not initialized");

    wrapper::close_notification_by_id(
        notifications,
        &CHANNEL,
        id,
        reason,
    )
}

pub fn invoke_notification_action(
    id: u32,
    action_key: &str,
) {
    wrapper::emit_notification_action_invoked(id, action_key);
}