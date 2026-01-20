use super::wrapper::Notification;

pub const NOTIFICATIONS_DBUS_BUS: &str = "org.freedesktop.Notifications";
pub const NOTIFICATIONS_DBUS_OBJECT: &str = "/org/freedesktop/Notifications";

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum BusEvent {
    NotificationAdded(Notification), // id, new notification
    NotificationUpdated(u32, Notification), // id, updated notification
    NotificationClosed(u32), // id, reason
}