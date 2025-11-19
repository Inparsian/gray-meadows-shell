pub mod mpris_player;
mod mpris_metadata;
mod mpris_dbus;

use std::{rc::Rc, sync::LazyLock};
use dbus::message::{MatchRule, MessageType};
use futures_signals::{signal::SignalExt, signal_vec::{SignalVecExt, VecDiff}};

use crate::{dbus::start_monitoring, ipc};

const MPRIS_DBUS_PREFIX: &str = "org.mpris.MediaPlayer2";
const MPRIS_DBUS_PATH: &str = "/org/mpris/MediaPlayer2";

#[derive(Default, Clone)]
pub struct Mpris {
    pub players: futures_signals::signal_vec::MutableVec<mpris_player::MprisPlayer>,
    pub default_player: futures_signals::signal::Mutable<usize>
}

pub static MPRIS: LazyLock<Mpris> = LazyLock::new(Mpris::default);

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
                VecDiff::Push {..} => if MPRIS.players.lock_ref().len() == 1 {
                    run_callback();
                },

                VecDiff::UpdateAt { index, .. } |
                VecDiff::RemoveAt { index } => if index == MPRIS.default_player.get() {
                    run_callback();
                },
                
                VecDiff::Pop {} | VecDiff::Clear {} => run_callback(),

                _ => {}
            }

            async {}
        })
    };

    let default_player_future = MPRIS.default_player.signal().for_each(move |index| {
        let callback = callback.clone();
        
        gtk4::glib::source::idle_add_local_once(move || callback(index));
        
        async {}
    });

    gtk4::glib::spawn_future_local(players_future);
    gtk4::glib::spawn_future_local(default_player_future);
}

pub fn activate() {
    // Get our initial list of MPRIS players
    let connection = dbus::blocking::SyncConnection::new_session().expect("Failed to connect to D-Bus");
    let proxy = connection.with_proxy(
        "org.freedesktop.DBus", 
        "/org/freedesktop/DBus", 
        std::time::Duration::from_millis(5000)
    );

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
    
    start_monitoring(rule, false, mpris_dbus::handle_master_message);

    // Monitor the MPRIS players for changes
    tokio::spawn(MPRIS.players.signal_vec_cloned().for_each(|change| {
        match change {
            VecDiff::RemoveAt {..} | VecDiff::Pop {} | VecDiff::Clear {} => assert_default_player(),
            _ => {}
        }

        async {}
    }));

    // Monitor IPC commands for controlling MPRIS players
    ipc::listen_for_messages(move |message| {
        match message.as_str() {
            "mpris_next" => { let _ = crate::singletons::mpris::get_default_player().map(|p| p.next()); },
            "mpris_previous" => { let _ = crate::singletons::mpris::get_default_player().map(|p| p.previous()); },
            "mpris_play_pause" => { let _ = crate::singletons::mpris::get_default_player().map(|p| p.play_pause()); },
            "mpris_play" => { let _ = crate::singletons::mpris::get_default_player().map(|p| p.play()); },
            "mpris_pause" => { let _ = crate::singletons::mpris::get_default_player().map(|p| p.pause()); },
            "mpris_volume_up" => { let _ = crate::singletons::mpris::get_default_player().map(|p| p.adjust_volume(0.05)); },
            "mpris_volume_down" => { let _ = crate::singletons::mpris::get_default_player().map(|p| p.adjust_volume(-0.05)); },
            _ => {},
        }
    });
}