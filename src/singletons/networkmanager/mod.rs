pub mod proxy;
pub mod wrapper;
pub mod enums;
pub mod bus;

use std::time::Duration;
use dbus::{channel::MatchingReceiver, message::{MatchRule, MessageType}};

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
    std::thread::spawn(|| {
        let connection = dbus::blocking::Connection::new_system()
            .expect("Failed to connect to D-Bus");

        let proxy = connection.with_proxy("org.freedesktop.DBus", "/org/freedesktop/DBus", Duration::from_millis(5000));

        let rule = MatchRule::new()
            .with_type(MessageType::Signal);

        let become_monitor_result: Result<(), dbus::Error> =
            proxy.method_call("org.freedesktop.DBus.Monitoring", "BecomeMonitor", (vec![rule.match_str()], 0_u32));

        match become_monitor_result {
            Ok(()) => {
                println!("Successfully became a monitor for NetworkManager signals");

                // Listen for signals
                connection.start_receive(rule, Box::new(|msg, _| {
                    handle_nm_signal_message(&msg);
                    true
                }));
            },

            Err(err) => {
                eprintln!("Failed to become a monitor: {}", err);
                eprintln!("Falling back to eavesdropping...");

                let eavesdrop_rule = {
                    let mut rule = rule.clone();
                    rule.eavesdrop = true;
                    rule
                };

                let add_eavesdrop_rule_result = connection.add_match(eavesdrop_rule, |(), _, msg| {
                    handle_nm_signal_message(msg);
                    true
                });

                match add_eavesdrop_rule_result {
                    Ok(_) => println!("Now eavesdropping NetworkManager signals"),
                    Err(e) => {
                        eprintln!("Failed to add eavesdropping match rule: {}", e);
                        eprintln!("Trying without eavesdropping...");

                        let add_rule_result = connection.add_match(rule, |(), _, msg| {
                            handle_nm_signal_message(msg);
                            true
                        });

                        match add_rule_result {
                            Ok(_) => println!("Now monitoring NetworkManager signals without eavesdropping"),
                            Err(e) => {
                                eprintln!("Failed to add match rule: {}", e);
                                eprintln!("Unable to monitor NetworkManager signals.");
                            }
                        }
                    }
                }
            },
        }

        loop {
            connection.process(Duration::from_millis(1000)).unwrap();
        }
    });
}