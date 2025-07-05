use dbus::arg::RefArg;

use crate::singletons::tray::wrapper::sn_item::StatusNotifierItem;

pub const WATCHER_DBUS_BUS: &str = "org.kde.StatusNotifierWatcher";
pub const WATCHER_DBUS_OBJECT: &str = "/StatusNotifierWatcher";
pub const ITEM_DBUS_BUS: &str = "org.kde.StatusNotifierItem";
pub const ITEM_DBUS_OBJECT: &str = "/StatusNotifierItem";
pub const DBUSMENU_BUS: &str = "com.canonical.dbusmenu";
// DBUSMENU_OBJECT is not a constant here because it can vary based on the item
pub const FREEDESKTOP_DBUS_BUS: &str = "org.freedesktop.DBus";
pub const FREEDESKTOP_DBUS_OBJECT: &str = "/org/freedesktop/DBus";

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum BusEvent {
    /// A StatusNotifierItem was registered on the D-Bus.
    ItemRegistered(StatusNotifierItem),

    /// A StatusNotifierItem was updated on the D-Bus.
    ItemUpdated(String, StatusNotifierItem),

    /// A StatusNotifierItem was unregistered from the D-Bus.
    ItemUnregistered(StatusNotifierItem),
}

pub fn make_key_value_pairs(value: &dyn RefArg) -> Vec<(String, &dyn RefArg)> {
    let mut pairs = Vec::new();

    if let Some(iter) = value.as_iter() {
        // Every odd entry is a key, every even entry is a value
        let mut enumerate = iter.enumerate();
        while let Some((i, entry)) = enumerate.next() {
            if i % 2 == 0 {
                if let Some(key) = entry.as_str() {
                    if let Some(value) = enumerate.next() {
                        pairs.push((key.to_string(), value.1));
                    }
                }
            }
        }
    }

    pairs
}