pub mod proxy;
pub mod wrapper;
pub mod enums;
pub mod bus;

use dbus::message::{MatchRule, MessageType};

#[allow(dead_code)]
pub fn handle_nm_signal_message(_msg: &dbus::Message) {
    todo!("Handle NetworkManager signal messages here");
}

#[allow(dead_code, unused_variables)]
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
                debug!(?device_type, ?driver, "NetworkManager device");
            }
        }
        Err(e) => {
            error!(%e, "Error retrieving devices");
        }
    }

    // NetworkManager monitor testing
    let rule = MatchRule::new()
        .with_type(MessageType::Signal);

    //start_monitoring(rule, true, handle_nm_signal_message);
}