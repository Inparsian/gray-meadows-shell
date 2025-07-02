use tokio::sync::broadcast;
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

use crate::singletons::tray::{wrappers::{
    status_notifier_watcher::{obtain_proxy, StatusNotifierWatcher}
}};

pub static TRAY_CONNECTION: OnceCell<Connection> = OnceCell::new();
static SENDER: Lazy<broadcast::Sender<TrayEvent>> = Lazy::new(|| {
    broadcast::channel(1).0
});

#[derive(Debug, Clone)]
pub enum TrayEvent {
    Register(String),
    Update(String, String),
    Unregister(String),
}

pub async fn activate() -> Result<()> {
    let watcher = StatusNotifierWatcher::new();
    let connection = Connection::session().await?;

    connection.object_server()
        .at("/StatusNotifierWatcher", watcher.clone())
        .await?;

    connection.request_name("org.kde.StatusNotifierWatcher").await?;

    TRAY_CONNECTION.set(connection).unwrap();

    // Watch for items being registered, updated and unregistered
    let watcher_proxy = obtain_proxy().await?;
    let mut register_receiver = watcher_proxy.receive_status_notifier_item_registered().await?;
    let mut update_receiver = watcher_proxy.receive_status_notifier_item_updated().await?;
    let mut unregister_receiver = watcher_proxy.receive_status_notifier_item_unregistered().await?;

    tokio::spawn(async move { loop {
        tokio::select! {
            Some(signal) = register_receiver.next() => {
                let service = signal.args().unwrap().service.to_owned();

                emit(TrayEvent::Register(service.clone())).await;
            },

            Some(signal) = update_receiver.next() => {
                let args = signal.args().unwrap();
                let service = args.service.to_owned();
                let member = args.member.to_owned();

                emit(TrayEvent::Update(service, member)).await;
            },

            Some(signal) = unregister_receiver.next() => {
                let service = signal.args().unwrap().service.to_owned();

                emit(TrayEvent::Unregister(service)).await;
            }
        }
    }});

    Ok(())
}

pub async fn get_items() -> Result<Vec<String>> {
    let watcher_proxy = obtain_proxy().await?;
    let items = watcher_proxy.registered_status_notifier_items().await?;

    Ok(items)
}

pub async fn subscribe() -> broadcast::Receiver<TrayEvent> {
    SENDER.subscribe()
}

async fn emit(event: TrayEvent) {
    let _ = SENDER.send(event);
}