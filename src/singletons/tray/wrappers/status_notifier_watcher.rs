use zbus::{fdo::DBusProxy, interface, names::MemberName, object_server::SignalEmitter, Connection, Error, Result};
use futures_lite::stream::StreamExt;

use crate::singletons::tray::{
    proxies::notifier_watcher_proxy::StatusNotifierWatcherProxy,
    wrappers::status_notifier_item::{get_raw_owner, obtain_status_notifier_item_proxy},
    TRAY_CONNECTION
};

#[derive(Debug, Clone, Default)]
pub struct StatusNotifierWatcher {
    is_status_notifier_host_registered: bool,
    protocol_version: i32,
    registered_status_notifier_items: Vec<String>,
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

        self.registered_status_notifier_items.push(service.to_owned());

        emitter.status_notifier_item_registered(item_path).await.unwrap_or_else(|e| {
            eprintln!("Failed to emit status_notifier_item_registered signal: {}", e);
        });

        tokio::spawn(watch_item_owner(service.to_owned()));
        tokio::spawn(watch_item_props(get_raw_owner(item_path)));
    }

    pub async fn unregister_status_notifier_item(
        &mut self,
        service: &str,
        #[zbus(signal_emitter)]
        emitter: SignalEmitter<'_>
    ) {
        let item_path = &format!("{service}/StatusNotifierItem");

        self.registered_status_notifier_items.retain(|owner| *owner != get_raw_owner(item_path));
        self.registered_status_notifier_items.shrink_to_fit();

        emitter.status_notifier_item_unregistered(item_path).await.unwrap_or_else(|e| {
            eprintln!("Failed to emit status_notifier_item_unregistered signal: {}", e);
        });
    }

    pub async fn update_status_notifier_item(
        &self,
        service: &str,
        member: &str,
        #[zbus(signal_emitter)]
        emitter: SignalEmitter<'_>
    ) {
        emitter.status_notifier_item_updated(service, member).await.unwrap_or_else(|e| {
            eprintln!("Failed to emit status_notifier_item_updated signal: {}", e);
        });
    }

    #[zbus(signal)]
    async fn status_notifier_host_registered(emitter: &SignalEmitter<'_>) -> Result<()>;

    #[zbus(signal)]
    async fn status_notifier_host_unregistered(emitter: &SignalEmitter<'_>) -> Result<()>;

    #[zbus(signal)]
    async fn status_notifier_item_registered(emitter: &SignalEmitter<'_>, service: &str) -> Result<()>;

    #[zbus(signal)]
    async fn status_notifier_item_updated(emitter: &SignalEmitter<'_>, service: &str, member: &str) -> Result<()>;

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
    pub fn registered_status_notifier_items(&self) -> Vec<String> {
        self.registered_status_notifier_items.clone()
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

async fn watch_item_props(owner: String) -> Result<()> {
    let watcher_proxy = obtain_proxy().await?;
    let item_proxy = obtain_status_notifier_item_proxy(&owner).await?;

    let mut unregistered_receiver = watcher_proxy.receive_status_notifier_item_unregistered().await?;
    let mut props_receiver = item_proxy.inner().receive_all_signals().await?;

    loop {
        tokio::select! {
            Some(change) = props_receiver.next() => {
                watcher_proxy.update_status_notifier_item(
                    &owner,
                    change.header().member().unwrap_or(&MemberName::try_from("unknown").unwrap()).as_str(),
                ).await.unwrap_or_else(|e| {
                    eprintln!("Failed to update status notifier item: {}", e);
                });
            },

            Some(signal) = unregistered_receiver.next() => {
                let service = signal.args().unwrap().service;

                if service == format!("{owner}/StatusNotifierItem") {
                    break Ok(());
                }
            }
        }
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