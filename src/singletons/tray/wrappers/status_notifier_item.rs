use zbus::Result;
use crate::singletons::tray::proxies::notifier_item_proxy::StatusNotifierItemProxy;

/// Removes the "/StatusNotifierItem" at the end of an item owner
pub fn get_raw_owner(item_path: &str) -> String {
    item_path.trim_end_matches("/StatusNotifierItem").to_string()
}

pub async fn obtain_status_notifier_item_proxy<'a>(service: &'a str) -> Result<StatusNotifierItemProxy<'a>> {
    if let Some(connection) = crate::singletons::tray::TRAY_CONNECTION.get() {
        StatusNotifierItemProxy::builder(connection)
            .destination(service)?
            .path("/StatusNotifierItem")?
            .build()
            .await
    } else {
        Err(zbus::Error::Failure("Tray not initialized".into()))
    }
}