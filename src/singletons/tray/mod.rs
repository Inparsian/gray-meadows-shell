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

pub async fn activate() -> Result<()> {
    let watcher = StatusNotifierWatcher::new();
    let connection = Connection::session().await?;

    connection.object_server()
        .at("/StatusNotifierWatcher", watcher)
        .await?;

    connection.request_name("org.kde.StatusNotifierWatcher").await?;

    loop {
        std::future::pending::<()>().await;
    }
}