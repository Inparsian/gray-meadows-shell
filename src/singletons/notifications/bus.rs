use super::wrapper::Notification;

pub const NOTIFICATIONS_DBUS_BUS: &str = "org.freedesktop.Notifications";
pub const NOTIFICATIONS_DBUS_OBJECT: &str = "/org/freedesktop/Notifications";

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum BusEvent {
    NotificationAdded(u32, Notification),   // id, new notification
    NotificationUpdated(u32, Notification), // id, updated notification
    NotificationClosed(u32), // id, reason
    ActionInvoked(u32, String),    // id, action_key
}