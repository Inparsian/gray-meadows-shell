use zbus::{Connection, Result};
use crate::singletons::tray::proxies::notifier_item_proxy::StatusNotifierItemProxy;

#[derive(Clone, Debug)]
pub struct StatusNotifierItem {
    pub owner: String
}

impl StatusNotifierItem {
    pub fn new(owner: String) -> Self {
        Self { owner }
    }
}

/// Removes the "/StatusNotifierItem" at the end of an item owner
pub fn get_raw_owner(item_path: String) -> String {
    item_path.trim_end_matches("/StatusNotifierItem").to_string()
}

pub async fn obtain_status_notifier_item_proxy(service: &str) -> Result<StatusNotifierItemProxy> {
    let connection = Connection::session().await?;
    
    StatusNotifierItemProxy::builder(&connection)
        .destination(service)?
        .path("/StatusNotifierItem")?
        .build()
        .await
}