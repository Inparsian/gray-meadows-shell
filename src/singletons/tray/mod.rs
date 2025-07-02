use once_cell::sync::OnceCell;
use zbus::{Connection, Result};

mod proxies {
    pub mod dbus_menu_proxy;
    pub mod notifier_item_proxy;
    pub mod notifier_watcher_proxy;
}

mod wrappers {
    pub mod status_notifier_watcher;
}

use crate::singletons::tray::wrappers::status_notifier_watcher::StatusNotifierWatcher;

pub static TRAY_CONNECTION: OnceCell<Connection> = OnceCell::new();
pub static STATUS_NOTIFIER_WATCHER: OnceCell<StatusNotifierWatcher> = OnceCell::new();

pub async fn activate() -> Result<()> {
    let watcher = StatusNotifierWatcher::new();
    let connection = Connection::session().await?;

    connection.object_server()
        .at("/StatusNotifierWatcher", watcher.clone())
        .await?;

    connection.request_name("org.kde.StatusNotifierWatcher").await?;

    TRAY_CONNECTION.set(connection).unwrap();
    STATUS_NOTIFIER_WATCHER.set(watcher).unwrap();

    loop {
        std::future::pending::<()>().await;
    }
}