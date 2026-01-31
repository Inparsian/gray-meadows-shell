use std::sync::{Arc, RwLock};
use async_broadcast::Receiver;
use dbus::{blocking, message::MatchRule, MessageType};
use dbus_crossroads::{Crossroads, IfaceToken};

use crate::dbus::start_monitoring;
use crate::utils::broadcast::BroadcastChannel;
use super::sn_item::StatusNotifierItem;
use super::super::bus::{self, BusEvent};
use super::super::proxy::{self, watcher::OrgKdeStatusNotifierWatcher};

/// https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/StatusNotifierWatcher/
#[derive(Debug, Clone)]
pub struct StatusNotifierWatcher {
    items: Arc<RwLock<Vec<StatusNotifierItem>>>,
    channel: BroadcastChannel<BusEvent>,
    // We don't implement the rest of the fields for simplicity's sake.
}

impl OrgKdeStatusNotifierWatcher for StatusNotifierWatcher {
    fn register_status_notifier_item(&mut self, service: String, sender: Option<dbus::strings::BusName>) -> Result<(), dbus::MethodErr> {
        let item = if service[0..1].eq(":") {
            StatusNotifierItem::new(service)
        } else if let Some(sender) = sender {
            // This item is intending to register a custom item bus name, register the sender instead
            StatusNotifierItem::new_with_path(
                sender.to_string(),
                service,
            )
        } else {
            return Err(dbus::MethodErr::failed(&"Invalid service name or sender"));
        };

        if self.items.read().is_ok_and(|items| items.iter().any(|existing_item| existing_item.service == item.service)) {
            Err(dbus::MethodErr::failed(&"Item already registered"))
        } else if let Ok(mut items) = self.items.write() {
            items.push(item.clone());
            self.channel.send_blocking(BusEvent::ItemRegistered(item));
            Ok(())
        } else {
            Err(dbus::MethodErr::failed(&"Failed to acquire write lock on items"))
        }
    }
    
    fn register_status_notifier_host(&mut self, _service: String) -> Result<(), dbus::MethodErr> {
        Ok(())
    }

    fn registered_status_notifier_items(&self) -> Result<Vec<String>, dbus::MethodErr> {
        Ok(self.items.try_read().map(|items| items.iter().map(|item| item.service.clone()).collect()).unwrap_or_default())
    }

    fn is_status_notifier_host_registered(&self) -> Result<bool, dbus::MethodErr> {
        // Here we set this to true immediately so StatusNotifierItems know they can use
        // our custom protocol instead of falling back to Freedesktop's protocol.
        Ok(true)
    }

    fn protocol_version(&self) -> Result<i32, dbus::MethodErr> {
        Ok(1)
    }
}

impl Default for StatusNotifierWatcher {
    fn default() -> Self {
        let channel = BroadcastChannel::new(10);

        StatusNotifierWatcher {
            items: Arc::new(RwLock::new(Vec::new())),
            channel,
        }
    }
}

impl StatusNotifierWatcher {
    /// Subscribes to events from this watcher.
    /// 
    /// You should call this before calling `serve`, so you won't miss any events and you 
    /// have the receiver you need before this object is consumed.
    pub fn subscribe(&self) -> Receiver<BusEvent> {
        self.channel.subscribe()
    }

    /// Retrieves an Arc to the items list's RwLock.
    /// 
    /// You should call this before calling `serve`, so you can access the items list
    /// while the watcher is serving.
    pub fn items(&self) -> Arc<RwLock<Vec<StatusNotifierItem>>> {
        Arc::clone(&self.items)
    }

    /// Serves clients forever on a new D-Bus session, consuming this watcher.
    /// 
    /// This is permanently blocking, you will probably want to run this in a separate thread:
    /// 
    /// ```rust
    /// std::thread::spawn(move || watcher.serve());
    /// ```
    pub fn serve(self) -> Result<(), dbus::Error> {
        let connection = blocking::Connection::new_session()?;

        let mut crossroads = Crossroads::new();
        let watcher_token: IfaceToken<StatusNotifierWatcher> = proxy::watcher::register_org_kde_status_notifier_watcher(&mut crossroads);

        // Register the item monitor before we move ownership of the watcher
        // to the crossroads.
        self.monitor_items()?;

        crossroads.insert(bus::WATCHER_DBUS_OBJECT, &[watcher_token], self);
        connection.request_name(
            bus::WATCHER_DBUS_BUS,
            false,
            true,
            false
        )?;

        crossroads.serve(&connection)
    }

    /// Creates a new D-Bus connection on a separate thread to monitor items on the bus.
    ///
    /// Meant to be used internally in serve.
    fn monitor_items(&self) -> Result<(), dbus::Error> {
        let rule = MatchRule::new()
            .with_type(MessageType::Signal);

        start_monitoring(rule, false, {
            let items = self.items();
            let channel = self.channel.clone();

            move |msg: &dbus::Message| {
                if let Some(member) = msg.member() {
                    let member = member.to_string();

                    // Handle item unregistration signals
                    if member == "NameOwnerChanged" {
                        let (_, old_owner, new_owner) = msg.get3::<String, String, String>();

                        if let (Some(old_owner), Some(new_owner)) = (old_owner, new_owner)
                            && new_owner.is_empty() && !old_owner.is_empty()
                            && let Some(item) = items.write()
                                .map_or(None, |mut writer| {
                                    writer.iter()
                                        .position(|item| item.service == old_owner)
                                        .map(|index| writer.remove(index))
                                })
                        {
                            channel.send_blocking(BusEvent::ItemUnregistered(item));
                        }
                    }
                    
                    // Handle update signals from items
                    else {
                        let service = msg.sender().unwrap().to_string();

                        // Clone the item and update that first, then write to the item. This prevents DoS by dbus abuse
                        if let Some(updated_item) = items.read().map_or(
                            None, 
                            |reader| reader.iter().find(|item| item.service == service).map(|item| {
                                let mut updated_item = item.clone();
                                updated_item.pass_update(&member);
                                updated_item
                            })
                        ) {
                            if let Ok(mut writer) = items.write()
                            && let Some(original_item) = writer.iter_mut()
                                .find(|item| item.service == service)
                            {
                                original_item.update_from(&updated_item);
                            }

                            channel.send_blocking(BusEvent::ItemUpdated(member, updated_item));
                        }
                    }
                }
            }
        });

        Ok(())
    }
}