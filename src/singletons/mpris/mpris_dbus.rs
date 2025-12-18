use std::time::Duration;
use dbus::arg::{self, Append, IterAppend, RefArg};
use dbus::blocking::{Connection, stdintf::org_freedesktop_dbus::Properties as _, BlockingSender as _};
use dbus::strings::BusName;
use dbus::{Error, Message};

use super::{MPRIS, MPRIS_DBUS_PATH, MPRIS_DBUS_PREFIX};
use super::mpris_player::{self, MprisPlayer};

pub fn handle_master_message(msg: &Message) {
    if let Some(member) = msg.member() {
        let member = member.trim();

        if member == "NameOwnerChanged" {
            let (bus, _, new_owner) = msg.get3::<String, String, String>();

            if let (Some(bus), Some(new_owner)) = (bus, new_owner) {
                if bus.starts_with(MPRIS_DBUS_PREFIX) {
                    MPRIS.players.lock_mut().retain(|player| player.bus != bus);

                    if !new_owner.is_empty() {
                        let player = mpris_player::MprisPlayer::new(bus, new_owner);
                        MPRIS.players.lock_mut().push_cloned(player);
                    }
                }
            } else {
                eprintln!("Failed to parse NameOwnerChanged message: {:?}", msg);
            }
        }

        else if let Some(path) = msg.path()
            && path.starts_with(MPRIS_DBUS_PATH) 
            && let Some(sender) = msg.sender()
        {
            let mut players_mut = MPRIS.players.lock_mut();
            let player_index = players_mut.iter()
                .position(|p| sender == BusName::new(&p.owner).unwrap())
                .unwrap_or(usize::MAX); // Default to an impossible index if not found

            if let Some(player) = players_mut.get(player_index) {
                let mut player = player.clone();

                match member {
                    "PropertiesChanged" => player.properties_changed(msg),
                    "Seeked" => player.seeked(msg),
                    _ => eprintln!("Unknown MPRIS signal member: {}", member),
                }

                players_mut.set_cloned(player_index, player);
            } else {
                eprintln!("Failed to find MPRIS player for owner: {}", sender);
            }
        }
    }
}

fn ready_dbus_message(player: &MprisPlayer, method: &str) -> Result<(Connection, Message), Error> {
    match Message::new_method_call(
        &*player.bus,
        "/org/mpris/MediaPlayer2",
        "org.mpris.MediaPlayer2.Player",
        method
    ) {
        Ok(message) => {
            let connection = Connection::new_session()
                .map_err(|e| Error::new_failed(&format!("Failed to connect to D-Bus: {}", e)))?;

            Ok((connection, message))
        },

        Err(err) => Err(Error::new_failed(&format!("Failed to create D-Bus message for method '{}': {}", method, err)))
    }
}

pub fn get_dbus_property<T>(player: &MprisPlayer, property: &str) -> Result<T, Error>
where
    T: for<'b> arg::Get<'b> + 'static + RefArg
{
    let connection = Connection::new_session()?;
    let proxy = connection.with_proxy(
        player.bus.clone(),
        "/org/mpris/MediaPlayer2",
        Duration::from_secs(5)
    );

    proxy.get("org.mpris.MediaPlayer2.Player", property)
        .map_err(|err| Error::new_failed(&format!("Failed to get D-Bus property '{}': {}", property, err)))
}

pub fn set_dbus_property<T>(player: &MprisPlayer, property: &str, value: T) -> Result<(), Error>
where
    T: arg::Arg + arg::Append + RefArg
{
    let connection = Connection::new_session()?;
    let proxy = connection.with_proxy(
        player.bus.clone(),
        "/org/mpris/MediaPlayer2",
        Duration::from_secs(5)
    );

    proxy.set("org.mpris.MediaPlayer2.Player", property, value)
        .map_err(|err| Error::new_failed(&format!("Failed to set D-Bus property '{}': {}", property, err)))
}

pub fn run_dbus_method(player: &MprisPlayer, method: &str) -> Result<Message, Error> {
    let (connection, message) = ready_dbus_message(player, method)?;

    connection.send_with_reply_and_block(message, Duration::from_secs(5))
        .map_err(|e| Error::new_failed(&format!("Failed to send D-Bus message: {}", e)))
}

pub fn run_dbus_method_w_args<T>(player: &MprisPlayer, method: &str, args: &[T]) -> Result<Message, Error>
where
    T: Append,
{
    let (connection, mut message) = ready_dbus_message(player, method)?;

    let mut iter = IterAppend::new(&mut message);
    for arg in args {
        iter.append(arg);
    }

    connection.send_with_reply_and_block(message, Duration::from_secs(5))
        .map_err(|e| Error::new_failed(&format!("Failed to send D-Bus message with argument: {}", e)))
}