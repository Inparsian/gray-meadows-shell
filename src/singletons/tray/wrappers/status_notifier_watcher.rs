use zbus::{fdo::DBusProxy, interface, object_server::SignalEmitter, Connection, Result};
use futures_lite::stream::StreamExt;

use crate::singletons::tray::proxies::notifier_item_proxy::StatusNotifierItemProxy;

#[derive(Debug, Clone, Default)]
pub struct StatusNotifierWatcher {
    is_status_notifier_host_registered: bool,
    protocol_version: i32,
    registered_status_notifier_items: Vec<String>,
}

#[interface(name = "org.kde.StatusNotifierWatcher")]
impl StatusNotifierWatcher {
    pub async fn register_status_notifier_host(&self, _service: &str) {
        // Host processing is not required for our implementation, due to setting
        // is_status_notifier_host_registered to true by default.
    }

    pub async fn register_status_notifier_item(&mut self, service: &str) {
        let service_owned = service.to_string();
        println!("Registering status notifier item: {}", service_owned);

        // Watch the bus for the disappearance of this item.
        tokio::spawn({
            let service = service_owned.clone();

            async move {
                let connection = Connection::session().await.expect("Failed to connect to session bus");
                let dbus_proxy = DBusProxy::new(&connection).await;

                if let Ok(proxy) = dbus_proxy {
                    if let Ok(mut stream) = proxy.receive_name_owner_changed().await {
                        while let Some(next) = stream.next().await {
                            if let Ok(args) = next.args() {
                                let old_owner = args.old_owner();
                                let new_owner = args.new_owner();

                                if let (Some(old_owner), None) = (old_owner.as_ref(), new_owner.as_ref()) {
                                    if **old_owner == service {
                                        // The item has disappeared
                                        println!("StatusNotifierItem disappeared: {}", service);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                } else {
                    eprintln!("Failed to obtain DBusProxy for service: {}", service);
                }
            }
        });

        // Test code
        let proxy = obtain_status_notifier_item_proxy(&service_owned).await;
        if let Ok(proxy) = proxy {
            if let Ok(title) = proxy.title().await {
                println!("Title for {}: {}", service_owned, title);
            } else {
                eprintln!("Failed to get title for service: {}", service_owned);
            }
        } else {
            eprintln!("Failed to obtain StatusNotifierItemProxy for service: {}", service_owned);
        }
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

// temporary test function, this will be moved to a proper place later
async fn obtain_status_notifier_item_proxy(service: &str) -> Result<StatusNotifierItemProxy> {
    let connection = Connection::session().await?;
    
    StatusNotifierItemProxy::builder(&connection)
        .destination(service)?
        .path("/StatusNotifierItem")?
        .build()
        .await
}