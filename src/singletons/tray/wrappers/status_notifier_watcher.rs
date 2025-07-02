use zbus::{fdo::DBusProxy, interface, object_server::SignalEmitter, Connection, Error, Result};
use futures_lite::stream::StreamExt;

use crate::singletons::tray::{
    proxies::notifier_watcher_proxy::StatusNotifierWatcherProxy,
    wrappers::status_notifier_item::StatusNotifierItem,
    TRAY_CONNECTION
};

#[derive(Debug, Clone, Default)]
pub struct StatusNotifierWatcher {
    is_status_notifier_host_registered: bool,
    protocol_version: i32,
    registered_status_notifier_items: Vec<(String, StatusNotifierItem)>,
}

#[interface(name = "org.kde.StatusNotifierWatcher")]
impl StatusNotifierWatcher {
    pub async fn register_status_notifier_host(&self, _service: &str) {}

    pub async fn register_status_notifier_item(
        &mut self,
        service: &str,
        #[zbus(signal_emitter)]
        emitter: SignalEmitter<'_>
    ) {
        let item_path = &format!("{service}/StatusNotifierItem");

        self.registered_status_notifier_items.push((
            item_path.to_owned(),
            StatusNotifierItem::new(service.to_owned())
        ));

        emitter.status_notifier_item_registered(item_path).await.unwrap_or_else(|e| {
            eprintln!("Failed to emit status_notifier_item_registered signal: {}", e);
        });

        tokio::spawn(watch_item_owner(service.to_owned()));
    }

    pub async fn unregister_status_notifier_item(
        &mut self,
        service: &str,
        #[zbus(signal_emitter)]
        emitter: SignalEmitter<'_>
    ) {
        let item_path = &format!("{service}/StatusNotifierItem");

        self.registered_status_notifier_items.retain(|(s, _)| s != item_path);

        emitter.status_notifier_item_unregistered(item_path).await.unwrap_or_else(|e| {
            eprintln!("Failed to emit status_notifier_item_unregistered signal: {}", e);
        });
    }

    #[zbus(signal)]
    async fn status_notifier_host_registered(emitter: &SignalEmitter<'_>) -> Result<()>;

    #[zbus(signal)]
    async fn status_notifier_host_unregistered(emitter: &SignalEmitter<'_>) -> Result<()>;

    #[zbus(signal)]
    async fn status_notifier_item_registered(emitter: &SignalEmitter<'_>, service: &str) -> Result<()>;

    #[zbus(signal)]
    async fn status_notifier_item_unregistered(emitter: &SignalEmitter<'_>, service: &str) -> Result<()>;

    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> bool {
        self.is_status_notifier_host_registered
    }

    #[zbus(property)]
    fn protocol_version(&self) -> i32 {
        self.protocol_version
    }

    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> Vec<String> {
        // Dbus clients will expect Strings containing the owners of
        // the registered items, not the full items.
        self.registered_status_notifier_items.clone().into_iter()
            .map(|(service, _)| service)
            .collect()
    }
}

impl StatusNotifierWatcher {
    pub fn new() -> Self {
        Self {
            // Set this to true immediately so StatusNotifierItems know they can use our
            // custom protocol instead of falling back to Freedesktop's protocol.
            is_status_notifier_host_registered: true,
            ..Self::default()
        }
    }
}

pub async fn obtain_proxy<'a>() -> Result<StatusNotifierWatcherProxy<'a>> {
    if let Some(connection) = TRAY_CONNECTION.get() {
        StatusNotifierWatcherProxy::new(connection).await
    } else {
        Err(Error::Failure("Not initialized".into()))
    }
}

pub async fn obtain_emitter<'a>() -> Result<SignalEmitter<'a>> {
    if let Some(connection) = TRAY_CONNECTION.get() {
        SignalEmitter::new(connection, "/StatusNotifierWatcher")
    } else {
        Err(Error::Failure("Not initialized".into()))
    }
}

async fn watch_item_owner(service: String) -> Result<()> {
    let connection = Connection::session().await?;
    let dbus_proxy = DBusProxy::new(&connection).await?;
    let mut stream = dbus_proxy.receive_name_owner_changed().await?;

    while let Some(next) = stream.next().await {
        if let Ok(args) = next.args() {
            let old_owner = args.old_owner().as_ref();
            let new_owner = args.new_owner().as_ref();

            if let (Some(old_owner), None) = (old_owner, new_owner) {
                if **old_owner == service {
                    if let Ok(proxy) = obtain_proxy().await {
                        proxy.unregister_status_notifier_item(&service).await.unwrap_or_else(|e| {
                            eprintln!("Failed to unregister status notifier item: {}", e);
                        });
                    }

                    break;
                }
            }
        }
    }

    Ok(())
}