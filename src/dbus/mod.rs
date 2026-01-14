mod client;
mod crossroads;

use std::time::Duration;
use dbus::{message::MatchRule, channel::MatchingReceiver as _};

pub fn start_monitoring<F>(rule: MatchRule<'static>, system: bool, callback: F)
where
    F: Fn(&dbus::Message) + Send + 'static,
{
    std::thread::spawn(move || {
        if let Ok(connection) = if system {
            dbus::blocking::Connection::new_system()
        } else {
            dbus::blocking::Connection::new_session()
        } {
            let proxy = connection.with_proxy(
                "org.freedesktop.DBus", 
                "/org/freedesktop/DBus", 
                Duration::from_millis(5000)
            );

            let become_monitor_result: Result<(), dbus::Error> =
                proxy.method_call("org.freedesktop.DBus.Monitoring", "BecomeMonitor", (vec![rule.match_str()], 0_u32));

            match become_monitor_result {
                Ok(()) => {
                    info!(bus = if system { "system" } else { "session" }, "Successfully became a D-Bus monitor");

                    connection.start_receive(rule, Box::new(move |msg, _| {
                        callback(&msg);
                        true
                    }));
                },

                Err(err) => {
                    warn!(bus = if system { "system" } else { "session" }, %err, "Failed to become a D-Bus monitor, falling back to eavesdropping");

                    let callback_arc = std::sync::Arc::new(std::sync::Mutex::new(callback));

                    let eavesdrop_rule = {
                        let mut rule = rule.clone();
                        rule.eavesdrop = true;
                        rule
                    };

                    let add_eavesdrop_rule_result = connection.add_match(eavesdrop_rule, {
                        let callback_arc = callback_arc.clone();
                        move |(), _, msg| {
                            let callback_lock = callback_arc.lock().unwrap();
                            callback_lock(msg);
                            true
                        }
                    });

                    match add_eavesdrop_rule_result {
                        Ok(_) => info!(bus = if system { "system" } else { "session" }, "Now eavesdropping D-Bus signals"),
                        Err(e) => {
                            warn!(bus = if system { "system" } else { "session" }, %e, "Failed to add eavesdropping match rule, trying without eavesdropping");
                            
                            let add_rule_result = connection.add_match(rule, move |(), _, msg| {
                                let callback_lock = callback_arc.lock().unwrap();
                                callback_lock(msg);
                                true
                            });

                            match add_rule_result {
                                Ok(_) => info!(bus = if system { "system" } else { "session" }, "Now monitoring D-Bus signals without eavesdropping"),
                                Err(e) => {
                                    error!(bus = if system { "system" } else { "session" }, %e, "Failed to add match rule, unable to monitor D-Bus signals");
                                }
                            }
                        }
                    }
                },
            }

            loop {
                connection.process(Duration::from_millis(1000)).unwrap();
            }
        } else {
            error!("Failed to connect to D-Bus");
        }
    });
}