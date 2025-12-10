pub mod mpris_player;
mod mpris_metadata;
mod mpris_dbus;

use std::{rc::Rc, sync::LazyLock};
use dbus::message::{MatchRule, MessageType};
use futures_signals::{signal::SignalExt, signal_vec::{SignalVecExt, VecDiff}};

use crate::dbus::start_monitoring;
use crate::ipc;

const MPRIS_DBUS_PREFIX: &str = "org.mpris.MediaPlayer2";
const MPRIS_DBUS_PATH: &str = "/org/mpris/MediaPlayer2";

#[derive(Default, Clone)]
pub struct Mpris {
    pub players: futures_signals::signal_vec::MutableVec<mpris_player::MprisPlayer>,
    pub default_player: futures_signals::signal::Mutable<usize>
}

pub static MPRIS: LazyLock<Mpris> = LazyLock::new(Mpris::default);

fn assert_default_player() {
    if MPRIS.default_player.get() > MPRIS.players.lock_ref().len() - 1 {
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

    gtk4::glib::spawn_future_local({
        let callback = callback.clone();

        signal_vec_cloned!(MPRIS.players, (change) {
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
        })
    });

    gtk4::glib::spawn_future_local(signal!(MPRIS.default_player, (index) {
        let callback = callback.clone();
        
        gtk4::glib::source::idle_add_local_once(move || callback(index));
    }));
}

pub fn subscribe_to_player_list_changes<F>(callback: F)
where
    // vec diff and new list length
    F: Fn(VecDiff<mpris_player::MprisPlayer>, usize) + 'static,
{
    let callback = Rc::new(callback);

    gtk4::glib::spawn_future_local({
        let callback = callback.clone();

        signal_vec_cloned!(MPRIS.players, (change) {
            let run_callback = |difference| {
                let callback = callback.clone();

                gtk4::glib::source::idle_add_local_once(move || {
                    assert_default_player();
                    callback(difference, MPRIS.players.lock_ref().len() - 1);
                });
            };

            match change {
                VecDiff::Push {..} |
                VecDiff::RemoveAt {..} |
                VecDiff::Pop {} |
                VecDiff::Clear {} => run_callback(change),
                _ => {}
            }
        })
    });

    // add all current players
    let value = MPRIS.players.lock_ref().len();
    for index in 0..value {
        let callback = callback.clone();
        let player = MPRIS.players.lock_ref().get(index).cloned().unwrap();
        gtk4::glib::source::idle_add_local_once(move || callback(VecDiff::Push { value: player }, index));
    }
}

#[allow(clippy::option_if_let_else)]
pub fn with_default_player_mut<F, R>(func: F) -> Option<R>
where
    F: FnOnce(&mut mpris_player::MprisPlayer) -> R,
{
    assert_default_player();
    let mut players = MPRIS.players.lock_mut();
    let index = MPRIS.default_player.get();
    if let Some(player) = players.get(index) {
        let mut player = player.clone();
        let result = func(&mut player);
        players.set_cloned(index, player);
        Some(result)
    } else {
        None
    }
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
    tokio::spawn(signal_vec_cloned!(MPRIS.players, (change) {
        match change {
            VecDiff::RemoveAt {..} | VecDiff::Pop {} | VecDiff::Clear {} => assert_default_player(),
            _ => {}
        }
    }));

    // Monitor IPC commands for controlling MPRIS players
    ipc::listen_for_messages_local(move |message| {
        std::thread::spawn(move || match message.as_str() {
            "mpris_next" => { let _ = with_default_player_mut(|p| p.next()); },
            "mpris_previous" => { let _ = with_default_player_mut(|p| p.previous()); },
            "mpris_play_pause" => { let _ = get_default_player().map(|p| p.play_pause()); },
            "mpris_play" => { let _ = get_default_player().map(|p| p.play()); },
            "mpris_pause" => { let _ = get_default_player().map(|p| p.pause()); },
            "mpris_volume_up" => { let _ = get_default_player().map(|p| p.adjust_volume(0.05)); },
            "mpris_volume_down" => { let _ = get_default_player().map(|p| p.adjust_volume(-0.05)); },
            _ => {},
        });
    });
}