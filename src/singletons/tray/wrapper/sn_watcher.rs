use std::{sync::{Arc, Mutex}, time::Duration};
use dbus::{blocking, channel::MatchingReceiver, message::MatchRule, MessageType};
use dbus_crossroads::{Crossroads, IfaceToken};

use crate::singletons::tray::{bus::{self, BusEvent}, proxy::{self, watcher::OrgKdeStatusNotifierWatcher}};

use super::sn_item::StatusNotifierItem;

/// https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/StatusNotifierWatcher/
#[derive(Debug, Clone)]
pub struct StatusNotifierWatcher {
    items: Arc<Mutex<Vec<StatusNotifierItem>>>,
    sender: tokio::sync::broadcast::Sender<BusEvent>,
    // We don't implement the rest of the fields for simplicity's sake.
}

impl OrgKdeStatusNotifierWatcher for StatusNotifierWatcher {
    fn register_status_notifier_item(&mut self, service: String) -> Result<(), dbus::MethodErr> {
        let item = StatusNotifierItem::new(service);

        self.sender.send(BusEvent::ItemRegistered(item.clone())).unwrap();

        self.items.lock().unwrap().push(item);

        Ok(())
    }
    
    fn register_status_notifier_host(&mut self, _service: String) -> Result<(), dbus::MethodErr> {
        Ok(())
    }

    fn registered_status_notifier_items(&self) -> Result<Vec<String>, dbus::MethodErr> {
        Ok(self.items.lock().unwrap().iter().map(|item| item.service.clone()).collect())
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
        Self::new()
    }
}

impl StatusNotifierWatcher {
    /// Creates a new `StatusNotifierWatcher` with an empty list of items.
    pub fn new() -> Self {
        let (sender, _) = tokio::sync::broadcast::channel(100);

        StatusNotifierWatcher {
            items: Arc::new(Mutex::new(Vec::new())),
            sender,
        }
    }

    /// Subscribes to events from this watcher.
    /// 
    /// You should call this before calling `serve`, so you won't miss any events and you 
    /// have the receiver you need before this object is consumed.
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<BusEvent> {
        self.sender.subscribe()
    }

    /// Retrieves an Arc to the items list's Mutex.
    /// 
    /// You should call this before calling `serve`, so you can access the items list
    /// while the watcher is serving.
    pub fn items(&self) -> Arc<Mutex<Vec<StatusNotifierItem>>> {
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
        std::thread::spawn({
            let items = self.items();
            let sender = self.sender.clone();

            move || {
                let connection = blocking::Connection::new_session().expect("Failed to create D-Bus connection");
                let proxy = connection.with_proxy(
                    bus::FREEDESKTOP_DBUS_BUS,
                    bus::FREEDESKTOP_DBUS_OBJECT,
                    std::time::Duration::from_millis(5000),
                );

                // Start monitoring dbus for new MPRIS players + MPRIS player changes
                let rule = MatchRule::new()
                    .with_type(MessageType::Signal);

                let become_monitor_result: Result<(), dbus::Error> =
                    proxy.method_call("org.freedesktop.DBus.Monitoring", "BecomeMonitor", (vec![rule.match_str()], 0_u32));

                match become_monitor_result {
                    Ok(()) => {
                        // Listen for signals
                        connection.start_receive(rule, Box::new({
                            let items = items.clone();
                            
                            move |msg, _| {
                                if let Some(member) = msg.member() {
                                    let member = member.to_string();

                                    // Handle item unregistration signals
                                    if member == "NameOwnerChanged" {
                                        let (_, old_owner, new_owner) = msg.get3::<String, String, String>();

                                        if let (Some(old_owner), Some(new_owner)) = (old_owner, new_owner) {
                                            if new_owner.is_empty() && !old_owner.is_empty() {
                                                let mut lock = items.lock().unwrap();
                                            
                                                if let Some(index) = lock.iter().position(|item| item.service == old_owner) {
                                                    let item = lock.remove(index);
                                                
                                                    sender.send(BusEvent::ItemUnregistered(item)).unwrap();
                                                }
                                            }
                                        }
                                    }
                                    
                                    // Handle update signals from items
                                    else {
                                        let service = msg.sender().unwrap().to_string();

                                        if let Some(item) = items.lock().unwrap().iter_mut().find(|item| item.service == service) {
                                            match msg.path() {
                                                Some(path) if path == *bus::ITEM_DBUS_OBJECT => {
                                                    item.pass_update(&member);

                                                    sender.send(BusEvent::ItemUpdated(member, item.clone())).unwrap();
                                                },

                                                _ => {}
                                            }
                                        }
                                    }
                                }

                                true
                            }
                        }));
                    },
                
                    Err(err) => eprintln!("Failed to become a monitor for StatusNotifierWatcher: {}", err),
                }
            
                loop {
                    connection.process(Duration::from_secs(1)).unwrap();
                }
            }
        });

        Ok(())
    }
}