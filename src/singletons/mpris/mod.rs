pub mod mpris_player;
mod mpris_metadata;
mod mpris_dbus;

use std::{rc::Rc, time::Duration};
use dbus::{channel::MatchingReceiver, message::{MatchRule, MessageType}};
use futures_signals::{signal::SignalExt, signal_vec::{SignalVecExt, VecDiff}};
use once_cell::sync::Lazy;

const MPRIS_DBUS_PREFIX: &str = "org.mpris.MediaPlayer2";
const MPRIS_DBUS_PATH: &str = "/org/mpris/MediaPlayer2";

#[derive(Default, Clone)]
pub struct Mpris {
    pub players: futures_signals::signal_vec::MutableVec<mpris_player::MprisPlayer>,
    pub default_player: futures_signals::signal::Mutable<usize>
}

pub static MPRIS: Lazy<Mpris> = Lazy::new(Mpris::default);

fn assert_default_player() {
    if MPRIS.default_player.get() > MPRIS.players.lock_ref().len() {
        set_default_player(0);
    }
}

pub fn get_default_player() -> Option<mpris_player::MprisPlayer> {
    assert_default_player();
    MPRIS.players.lock_ref().get(MPRIS.default_player.get()).cloned()
}

pub fn set_default_player(index: usize) {
    if index < MPRIS.players.lock_ref().len() {
        MPRIS.default_player.set(index);
    } else if !MPRIS.players.lock_ref().is_empty() {
        eprintln!("Attempted to set default player to index {}, but only {} players are available.", index, MPRIS.players.lock_ref().len());
    } else {
        // None are available, just fallback to 0
        MPRIS.default_player.set(0);
    }
}

pub fn subscribe_to_default_player_changes<F>(callback: F)
where
    F: Fn(usize) + 'static,
{
    let callback = Rc::new(callback);

    let players_future = {
        let callback = callback.clone();

        MPRIS.players.signal_vec_cloned().for_each(move |change| {
            let run_callback = || {
                let callback = callback.clone();

                gtk4::glib::source::idle_add_local_once(move || {
                    assert_default_player();
                    callback(MPRIS.default_player.get());
                });
            };

            match change {
                // Do nothing if there's already more than one player
                // This is for if a player is instantiated when there's no players
                VecDiff::Push { value: _ } => if MPRIS.players.lock_ref().len() == 1 {
                    run_callback();
                },

                VecDiff::UpdateAt { index, value: _ } => if index == MPRIS.default_player.get() {
                    run_callback();
                },

                VecDiff::RemoveAt { index } => if index == MPRIS.default_player.get() {
                    run_callback();
                },
                
                VecDiff::Pop {} => run_callback(),
                VecDiff::Clear {} => run_callback(),

                _ => {}
            }

            async {}
        })
    };

    let default_player_future = {
        let callback = callback.clone();

        MPRIS.default_player.signal().for_each(move |index| {
            let callback = callback.clone();

            gtk4::glib::source::idle_add_local_once(move || callback(index));

            async {}
        })
    };

    gtk4::glib::MainContext::default().spawn_local(players_future);
    gtk4::glib::MainContext::default().spawn_local(default_player_future);
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

                    MPRIS.players.lock_mut().push_cloned(player);
                }
            }
        }

        // Start monitoring dbus for new MPRIS players + MPRIS player changes
        let rule = MatchRule::new()
            .with_type(MessageType::Signal);

        let become_monitor_result: Result<(), dbus::Error> =
            proxy.method_call("org.freedesktop.DBus.Monitoring", "BecomeMonitor", (vec![rule.match_str()], 0u32));

        match become_monitor_result {
            Ok(()) => {
                println!("Successfully became a monitor for MPRIS players");

                // Listen for signals
                connection.start_receive(rule, Box::new(|msg, _| {
                    mpris_dbus::handle_master_message(&msg);
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
    let future = MPRIS.players.signal_vec_cloned().for_each(|change| {
        match change {
            VecDiff::RemoveAt { index: _ } => assert_default_player(),
            VecDiff::Pop {} => assert_default_player(),
            VecDiff::Clear {} => assert_default_player(),

            _ => {}
        }

        async {}
    });

    tokio::spawn(future);
}