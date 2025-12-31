use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_broadcast::Receiver;
use dbus::blocking;
use dbus::arg::RefArg as _;
use dbus::channel::Sender as _;
use dbus_crossroads::{Crossroads, IfaceToken};

use crate::broadcast::BroadcastChannel;
use super::bus::{self, BusEvent};
use super::proxy::{self, OrgFreedesktopNotifications};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum NotificationCloseReason {
    Expired = 1,
    Dismissed = 2,
    ClosedByCall = 3,
    Undefined = 4,
}

#[derive(Debug, Clone)]
pub enum NotificationHint {
    Urgency(u8),
    Category(String),
}

#[derive(Debug, Clone)]
pub struct NotificationAction {
    pub id: String,
    pub localized_name: String,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u32,
    pub app_name: String,
    pub replaces_id: u32,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<NotificationAction>,
    pub hints: Vec<NotificationHint>,
    pub expire_timeout: i32,
}

#[derive(Debug, Clone)]
pub struct NotificationManager {
    id_counter: Arc<RwLock<u32>>,
    notifications: Arc<RwLock<HashMap<u32, Notification>>>,
    channel: BroadcastChannel<BusEvent>,
}

impl OrgFreedesktopNotifications for NotificationManager {
    fn notify(
        &mut self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: dbus::arg::PropMap,
        expire_timeout: i32,
    ) -> Result<u32, dbus::MethodErr> {
        let hints = hints.into_iter().filter_map(|(key, value)| {
            match key.as_str() {
                "urgency" => value.as_u64().map(|u| NotificationHint::Urgency(u as u8)),
                "category" => value.as_str().map(|s| NotificationHint::Category(s.to_owned())),
                _ => None,
            }
        }).collect();

        let actions = actions.as_chunks::<2>()
            .0
            .iter()
            .map(|chunk| NotificationAction {
                id: chunk[0].clone(),
                localized_name: chunk[1].clone(),
            })
            .collect();
            
        let mut notification = Notification {
            id: 0,
            app_name,
            replaces_id,
            app_icon,
            summary,
            body,
            actions,
            hints,
            expire_timeout,
        };

        let mut notifications = self.notifications.write()
            .map_err(|_| dbus::MethodErr::failed(&"Failed to acquire write lock on notifications"))?;

        if replaces_id > 0 {
            notification.id = replaces_id;
            notifications.insert(replaces_id, notification.clone());
            self.channel.send_blocking(BusEvent::NotificationUpdated(replaces_id, notification));
            Ok(replaces_id)
        } else {
            let id = {
                let mut id_counter = self.id_counter.write()
                    .map_err(|_| dbus::MethodErr::failed(&"Failed to acquire write lock on id_counter"))?;

                *id_counter += 1;
                *id_counter
            };

            notification.id = id;
            notifications.insert(id, notification.clone());
            self.channel.send_blocking(BusEvent::NotificationAdded(id, notification));
            Ok(id)
        }
    }

    fn close_notification(&mut self, id: u32) -> Result<(), dbus::MethodErr> {
        close_notification_by_id(
            &self.notifications,
            &self.channel,
            id,
            NotificationCloseReason::ClosedByCall,
        )
    }

    fn get_capabilities(&mut self) -> Result<Vec<String>, dbus::MethodErr> {
        Ok(vec![
            "body".to_owned(),
            "actions".to_owned(),
        ])
    }

    fn get_server_information(&mut self) -> Result<(String, String, String, String), dbus::MethodErr> {
        Ok((
            "Gray Meadows Shell".to_owned(),
            "Inparsian".to_owned(),
            "1.0".to_owned(),
            "1.2".to_owned(),
        ))
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        let channel = BroadcastChannel::new(10);

        NotificationManager {
            id_counter: Arc::new(RwLock::new(0)),
            notifications: Arc::new(RwLock::new(HashMap::new())),
            channel,
        }
    }
}

impl NotificationManager {
    /// Subscribes to notification events.
    pub fn subscribe(&self) -> Receiver<BusEvent> {
        self.channel.subscribe()
    }

    /// Retrieves an Arc to the notifications HashMap's RwLock.
    pub fn notifications(&self) -> Arc<RwLock<HashMap<u32, Notification>>> {
        Arc::clone(&self.notifications)
    }
    
    /// Serves clients forever on a new D-Bus session, consuming this manager.
    /// 
    /// This is permanently blocking, you will probably want to run this in a separate thread:
    /// 
    /// ```rust
    /// std::thread::spawn(move || manager.serve());
    /// ```
    pub fn serve(self) -> Result<(), dbus::Error> {
        let connection = blocking::Connection::new_session()?;

        let mut crossroads = Crossroads::new();
        let watcher_token: IfaceToken<NotificationManager> = proxy::register_org_freedesktop_notifications(&mut crossroads);

        crossroads.insert(bus::NOTIFICATIONS_DBUS_OBJECT, &[watcher_token], self);
        connection.request_name(
            bus::NOTIFICATIONS_DBUS_BUS,
            false,
            true,
            false
        )?;

        crossroads.serve(&connection)
    }
}

/// Emits a NotificationClosed signal for the given notification ID.
fn emit_notification_closed(id: u32, reason: u32) {
    let connection = blocking::Connection::new_session().expect("Failed to create D-Bus connection");
    let mut signal = dbus::Message::signal(
        &bus::NOTIFICATIONS_DBUS_OBJECT.into(),
        &bus::NOTIFICATIONS_DBUS_BUS.into(),
        &"NotificationClosed".into(),
    );

    signal.append_all(proxy::OrgFreedesktopNotificationsNotificationClosed {
        id,
        reason,
    });

    connection.send(signal).expect("Failed to send NotificationClosed signal");
}

/// Emits a NotificationActionInvoked signal for the given notification ID and action key.
pub(super) fn emit_notification_action_invoked(id: u32, action_key: &str) {
    let connection = blocking::Connection::new_session().expect("Failed to create D-Bus connection");
    let mut signal = dbus::Message::signal(
        &bus::NOTIFICATIONS_DBUS_OBJECT.into(),
        &bus::NOTIFICATIONS_DBUS_BUS.into(),
        &"ActionInvoked".into(),
    );

    signal.append_all(proxy::OrgFreedesktopNotificationsActionInvoked {
        id,
        action_key: action_key.to_owned(),
    });

    connection.send(signal).expect("Failed to send ActionInvoked signal");
}

/// Closes a notification by ID with the given reason.
/// This will emit the NotificationClosed signal with the reason.
pub(super) fn close_notification_by_id(
    notifications_ref: &Arc<RwLock<HashMap<u32, Notification>>>,
    channel: &BroadcastChannel<BusEvent>,
    id: u32,
    reason: NotificationCloseReason,
) -> Result<(), dbus::MethodErr> {
    let mut notifications = notifications_ref.write()
        .map_err(|_| dbus::MethodErr::failed(&"Failed to acquire write lock on notifications"))?;

    if notifications.remove(&id).is_some() {
        channel.send_blocking(BusEvent::NotificationClosed(id));
        emit_notification_closed(id, reason as u32);
        Ok(())
    } else {
        Err(dbus::MethodErr::failed(&"Notification ID not found"))
    }
}