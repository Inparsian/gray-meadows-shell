mod mpris_player;
mod mpris_metadata;

use std::time::Duration;
use dbus::{channel::MatchingReceiver, message::MatchRule, strings::BusName, Message};
use futures_signals::signal_vec::{SignalVecExt, VecDiff};
use once_cell::sync::Lazy;

const MPRIS_DBUS_PREFIX: &str = "org.mpris.MediaPlayer2";
const MPRIS_DBUS_PATH: &str = "/org/mpris/MediaPlayer2";

#[derive(Clone)]
pub struct Mpris {
    pub players: futures_signals::signal_vec::MutableVec<mpris_player::MprisPlayer>,
    pub default_player: futures_signals::signal::Mutable<usize>
}

pub static MPRIS: Lazy<Mpris> = Lazy::new(|| {
    Mpris {
        players: futures_signals::signal_vec::MutableVec::new(),
        default_player: futures_signals::signal::Mutable::new(0)
    }
});

fn assert_default_player() {
    if MPRIS.default_player.get() >= MPRIS.players.lock_ref().len() {
        MPRIS.default_player.set(0);
    }
}

#[allow(dead_code)]
pub fn get_default_player() -> Option<mpris_player::MprisPlayer> {
    assert_default_player();
    MPRIS.players.lock_ref().get(MPRIS.default_player.get()).cloned()
}

#[allow(dead_code)]
pub fn set_default_player(index: usize) {
    if index < MPRIS.players.lock_ref().len() {
        MPRIS.default_player.set(index);
    } else {
        eprintln!("Attempted to set default player to index {}, but only {} players are available.", index, MPRIS.players.lock_ref().len());
    }
}

fn handle_message(msg: &Message) {
    if let Some(member) = msg.member() {
        let member = member.trim();

        if &member == &"NameOwnerChanged" {
            let (bus, _, new_owner) = msg.get3::<String, String, String>();

            if let (Some(bus), Some(new_owner)) = (bus, new_owner) {
                if bus.starts_with(MPRIS_DBUS_PREFIX) {
                    MPRIS.players.lock_mut().retain(|player| player.bus != bus.clone().into());

                    if !new_owner.is_empty() {
                        let player = mpris_player::MprisPlayer::new(bus.clone(), new_owner.clone());
                        MPRIS.players.lock_mut().push(player);
                    }
                }
            } else {
                eprintln!("Failed to parse NameOwnerChanged message: {:?}", msg);
            }
        }

        else if let Some(path) = msg.path() {
            if path.starts_with(MPRIS_DBUS_PATH) && msg.msg_type() == dbus::message::MessageType::Signal {
                let sender: Option<BusName> = msg.sender();

                if let Some(sender) = sender {
                    let mut players_mut = MPRIS.players.lock_mut();
                    let player_index = players_mut.iter().position(|p| sender == p.owner.as_ref().into());

                    if let Some(player_index) = player_index {
                        let player = players_mut.get(player_index);

                        if let Some(player) = player {
                            let player = &mut player.clone();
                            
                            match member {
                                "PropertiesChanged" => player.properties_changed(msg),
                                "Seeked" => player.seeked(msg),
                                _ => eprintln!("Unknown MPRIS signal member: {}", member),
                            }

                            players_mut.set(player_index, *player);
                        } else {
                            eprintln!("Failed to find MPRIS player for owner: {}", sender);
                        }
                    }
                }
            }
        }
    }
}

pub fn activate() {
    std::thread::spawn(|| {
        // Monitor dbus for appearing and disappearing MPRIS players
        let connection = dbus::blocking::SyncConnection::new_session().expect("Failed to connect to D-Bus");
        let proxy = connection.with_proxy(
            "org.freedesktop.DBus", 
            "/org/freedesktop/DBus", 
            std::time::Duration::from_millis(5000)
        );

        // Get our initial list of MPRIS players
        let (names,): (Vec<String>,) = proxy.method_call("org.freedesktop.DBus", "ListNames", ()).unwrap();
        for name in names {
            if name.starts_with(MPRIS_DBUS_PREFIX) {
                // Get the owner for this player
                let owner_opt: Option<String> = proxy
                    .method_call("org.freedesktop.DBus", "GetNameOwner", (name.clone(),))
                    .map(|(owner,)| owner)
                    .ok();

                if let Some(owner) = owner_opt {
                    let player = mpris_player::MprisPlayer::new(name, owner);

                    MPRIS.players.lock_mut().push(player);
                }
            }
        }

        // Start monitoring dbus for new MPRIS players + MPRIS player changes
        let rule = MatchRule::new();
        let become_monitor_result: Result<(), dbus::Error> =
            proxy.method_call("org.freedesktop.DBus.Monitoring", "BecomeMonitor", (vec![rule.match_str()], 0u32));

        match become_monitor_result {
            Ok(()) => {
                println!("Successfully became a monitor for MPRIS players");

                // Listen for signals
                connection.start_receive(rule, Box::new(|msg, _| {
                    handle_message(&msg);
                    true
                }));
            },

            Err(err) => eprintln!("Failed to become a monitor: {}", err),
        }

        loop {
            connection.process(Duration::from_millis(1000)).unwrap();
        }
    });

    // Monitor the MPRIS players for changes
    let future = MPRIS.players.signal_vec().for_each(|change| {
        match change {
            VecDiff::InsertAt { index, value } => {
                println!("New MPRIS player added at index {}: {}", index, value.bus);
            },

            VecDiff::UpdateAt { index, value } => {
                println!("MPRIS player at index {} updated: {}", index, value.bus);
            },

            VecDiff::RemoveAt { index } => {
                assert_default_player();
                println!("MPRIS player removed at index {}", index);
            },

            VecDiff::Push { value } => {
                println!("New MPRIS player pushed to the end of the list: {}", value.bus);
            },

            VecDiff::Pop {} => {
                assert_default_player();
                println!("MPRIS player popped from the end of the list.");
            },

            _ => {
                println!("Unknown MPRIS player change: {:?}", change);
            }
        }

        async {}
    });

    tokio::spawn(future);
}