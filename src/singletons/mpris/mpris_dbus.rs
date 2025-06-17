use std::time::Duration;
use dbus::{arg::IterAppend, blocking::{BlockingSender, Connection}, strings::BusName, Error, Message};

use crate::singletons::mpris::{mpris_player::{self, MprisPlayer}, MPRIS, MPRIS_DBUS_PATH, MPRIS_DBUS_PREFIX};

pub fn handle_master_message(msg: &Message) {
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

fn ready_dbus_message(player: &MprisPlayer, method: &str) -> Result<(Connection, Message), Error> {
    let message = Message::new_method_call(
        &*player.bus,
        "/org/mpris/MediaPlayer2",
        "org.mpris.MediaPlayer2.Player",
        method
    );

    if let Ok(message) = message {
        let connection = dbus::blocking::Connection::new_session()
            .map_err(|e| Error::new_failed(&format!("Failed to connect to D-Bus: {}", e)))?;

        Ok((connection, message))
    } else {
        Err(Error::new_failed(&format!("Failed to create D-Bus message for method '{}': {}", method, message.err().unwrap())))
    }
}

pub fn run_dbus_method(player: &MprisPlayer, method: &str) -> Result<Message, Error> {
    let result = ready_dbus_message(player, method);

    if let Ok((connection, message)) = result {
        connection.send_with_reply_and_block(message, Duration::from_secs(5))
            .map_err(|e| Error::new_failed(&format!("Failed to send D-Bus message: {}", e)))
    } else {
        Err(Error::new_failed(&format!("Failed to connect to D-Bus: {}", result.err().unwrap())))
    }
}

pub fn run_dbus_method_i64(player: &MprisPlayer, method: &str, args: &[i64]) -> Result<Message, Error> {
    let (connection, mut message) = ready_dbus_message(player, method)?;

    let mut iter = IterAppend::new(&mut message);
    for arg in args {
        iter.append(arg);
    }

    connection.send_with_reply_and_block(message, Duration::from_secs(5))
        .map_err(|e| Error::new_failed(&format!("Failed to send D-Bus message with argument: {}", e)))
}