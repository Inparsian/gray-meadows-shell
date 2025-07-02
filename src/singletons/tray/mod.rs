use std::sync::Mutex;
use once_cell::sync::{OnceCell, Lazy};
use zbus::{Connection, Result};
use futures_lite::stream::StreamExt;

mod proxies {
    pub mod dbus_menu_proxy;
    pub mod notifier_item_proxy;
    pub mod notifier_watcher_proxy;
}

mod wrappers {
    pub mod status_notifier_watcher;
    pub mod status_notifier_item;
}

use crate::singletons::tray::wrappers::{status_notifier_item::{get_raw_owner, obtain_status_notifier_item_proxy}, status_notifier_watcher::{obtain_proxy, StatusNotifierWatcher}};

pub static TRAY_CONNECTION: OnceCell<Connection> = OnceCell::new();

// this is the list of items that will be exposed to the UI
pub static ITEMS: Lazy<Mutex<Vec<wrappers::status_notifier_item::StatusNotifierItem>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub async fn activate() -> Result<()> {
    let watcher = StatusNotifierWatcher::new();
    let connection = Connection::session().await?;

    connection.object_server()
        .at("/StatusNotifierWatcher", watcher.clone())
        .await?;

    connection.request_name("org.kde.StatusNotifierWatcher").await?;

    TRAY_CONNECTION.set(connection).unwrap();

    // Watch for items being registered and unregistered
    let watcher_proxy = obtain_proxy().await?;
    let mut register_receiver = watcher_proxy.receive_status_notifier_item_registered().await?;
    let mut unregister_receiver = watcher_proxy.receive_status_notifier_item_unregistered().await?;

    loop {
        tokio::select! {
            Some(signal) = register_receiver.next() => {
                let service = signal.args().unwrap().service.to_owned();

                println!("Item {service} registered");
                let item = wrappers::status_notifier_item::StatusNotifierItem::new(service.clone());
                ITEMS.lock().unwrap().push(item);
                
                // Watch this item for property update signal emissions
                tokio::spawn(watch_item(get_raw_owner(service)));
            },

            Some(signal) = unregister_receiver.next() => {
                let service = signal.args().unwrap().service;

                println!("Item {service} unregistered");
                ITEMS.lock().unwrap().retain(|item| item.owner != service);
            }
        }
    }
}

pub async fn watch_item(owner: String) -> Result<()> {
    let watcher_proxy = obtain_proxy().await?;
    let item_proxy = obtain_status_notifier_item_proxy(&owner).await?;

    let mut unregistered_receiver = watcher_proxy.receive_status_notifier_item_unregistered().await?;
    let mut props_receiver = item_proxy.inner().receive_all_signals().await?;

    println!("Now watching {owner}");

    loop {
        tokio::select! {
            Some(change) = props_receiver.next() => {
                println!("change: {:?}", change);
            },

            Some(signal) = unregistered_receiver.next() => {
                let service = signal.args().unwrap().service;

                if service == format!("{owner}/StatusNotifierItem") {
                    break;
                }
            }
        }
    }

    println!("No longer watching {owner}");

    Ok(())
}