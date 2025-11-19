pub mod proxy;
pub mod wrapper;
pub mod enums;
pub mod bus;

use dbus::{message::{MatchRule, MessageType}};

use crate::dbus::start_monitoring;

pub fn handle_nm_signal_message(msg: &dbus::Message) {
    if let Some(member) = msg.member() {
        if let Some(path) = msg.path() {
            if path.starts_with("/org/freedesktop/NetworkManager") {
                println!("Received NetworkManager signal: {} on path: {}", member, path);
            }
        }
    }
}

pub fn activate() {
    // DBus client proxy testing
    let connection = dbus::blocking::Connection::new_system()
        .expect("Failed to connect to system DBus");

    let proxy = connection.with_proxy(
        "org.freedesktop.NetworkManager",
        "/org/freedesktop/NetworkManager",
        std::time::Duration::from_millis(5000),
    );

    let devices = proxy::manager::OrgFreedesktopNetworkManager::devices(&proxy);
    match devices {
        Ok(devs) => {
            for dev in devs {
                let device_proxy = connection.with_proxy(
                    "org.freedesktop.NetworkManager",
                    dev,
                    std::time::Duration::from_millis(5000),
                );

                let device_type = <dyn proxy::device::OrgFreedesktopNetworkManagerDevice>::device_type(&device_proxy);
                let driver = <dyn proxy::device::OrgFreedesktopNetworkManagerDevice>::driver(&device_proxy);
                println!("Type: {:?}, Driver: {:?}", device_type, driver);
            }
        }
        Err(e) => {
            eprintln!("Error retrieving devices: {}", e);
        }
    }

    // NetworkManager monitor testing
    let rule = MatchRule::new()
        .with_type(MessageType::Signal);

    start_monitoring(rule, true, handle_nm_signal_message);
}