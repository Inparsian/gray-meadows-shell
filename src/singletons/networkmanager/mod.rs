pub mod proxy;
pub mod wrapper;
pub mod enums;
pub mod bus;

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
}