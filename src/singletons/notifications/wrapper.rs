use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_broadcast::Receiver;
use dbus::arg::RefArg as _;
use dbus::blocking;
use dbus_crossroads::{Crossroads, IfaceToken};

use crate::broadcast::BroadcastChannel;
use super::bus::{self, BusEvent};
use super::proxy::{self, OrgFreedesktopNotifications};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum NotificationHint {
    Urgency(u8),
    Category(String),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Notification {
    pub app_name: String,
    pub replaces_id: u32,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<String>,
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
            
        let notification = Notification {
            app_name,
            replaces_id,
            app_icon,
            summary,
            body,
            actions,
            hints,
            expire_timeout,
        };

        let id = {
            let mut id_counter = self.id_counter.write().map_err(|_| dbus::MethodErr::failed(&"Failed to acquire write lock on id_counter"))?;
            *id_counter += 1;
            *id_counter
        };

        let mut notifications = self.notifications.write()
            .map_err(|_| dbus::MethodErr::failed(&"Failed to acquire write lock on notifications"))?;

        if replaces_id > 0 {
            notifications.insert(replaces_id, notification.clone());
            self.channel.send_blocking(BusEvent::NotificationUpdated(replaces_id, notification));
        } else {
            notifications.insert(id, notification.clone());
            self.channel.send_blocking(BusEvent::NotificationAdded(id, notification));
        }

        Ok(id)
    }

    fn close_notification(&mut self, id: u32) -> Result<(), dbus::MethodErr> {
        let mut notifications = self.notifications.write()
            .map_err(|_| dbus::MethodErr::failed(&"Failed to acquire write lock on notifications"))?;

        if notifications.remove(&id).is_some() {
            self.channel.send_blocking(BusEvent::NotificationClosed(id));
            Ok(())
        } else {
            Err(dbus::MethodErr::failed(&"Notification ID not found"))
        }
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