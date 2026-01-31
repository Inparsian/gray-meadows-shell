use dbus::{arg::{self, RefArg}, Error, Message};

use super::{mpris_dbus, mpris_metadata};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    #[default] Stopped
}

impl PlaybackStatus {
    pub fn infer_from_string(status: &str) -> Self {
        match status {
            "Playing" => PlaybackStatus::Playing,
            "Paused" => PlaybackStatus::Paused,
            _ => PlaybackStatus::Stopped
        }
    }

    pub fn as_string(self) -> String {
        match self {
            PlaybackStatus::Playing => "Playing".to_owned(),
            PlaybackStatus::Paused => "Paused".to_owned(),
            PlaybackStatus::Stopped => "Stopped".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoopStatus {
    #[default] None,
    Track,
    Playlist
}

impl LoopStatus {
    pub fn infer_from_string(status: &str) -> Self {
        match status {
            "Track" => LoopStatus::Track,
            "Playlist" => LoopStatus::Playlist,
            _ => LoopStatus::None
        }
    }

    pub fn as_string(self) -> String {
        match self {
            LoopStatus::None => "None".to_owned(),
            LoopStatus::Track => "Track".to_owned(),
            LoopStatus::Playlist => "Playlist".to_owned()
        }
    }
}

// The common xesam and mpris metadata properties should be enough for most use cases,
// so this struct is here as an easy way to read metadata from the player. Paths are
// casted to strings for simplicity's sake.
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub track_id: Option<String>, // mpris:trackid
    pub length: Option<i64>, // mpris:length - in microseconds
    pub art_url: Option<String>, // mpris:artUrl
    pub album: Option<String>, // xesam:album
    pub artist: Option<Vec<String>>, // xesam:artist
    pub content_created: Option<String>, // xesam:contentCreated
    pub title: Option<String>, // xesam:title
}

#[derive(Debug, Clone, Default)]
pub struct MprisPlayer {
    pub bus: String,
    pub owner: String,
    pub playback_status: PlaybackStatus,
    pub loop_status: LoopStatus,
    pub rate: f64,
    pub shuffle: bool,
    pub metadata: Metadata,
    pub volume: f64,
    pub position: i64, // Time_In_Us (time in microseconds)
    pub minimum_rate: f64,
    pub maximum_rate: f64,
    pub can_go_next: bool,
    pub can_go_previous: bool,
    pub can_play: bool,
    pub can_pause: bool,
    pub can_seek: bool,
    pub can_control: bool
}

impl MprisPlayer {
    pub fn new(bus: String, owner: String) -> Self {
        let mut player = MprisPlayer {
            bus,
            owner,
            ..MprisPlayer::default()
        };
        
        // perform initial property sync
        player.sync_properties();

        player
    }

    fn get_metadata_property<T>(&self, key: &str) -> Result<T, Error> 
    where
        T: 'static + Clone + RefArg,
    {
        match mpris_dbus::get_dbus_property::<arg::PropMap>(self, "Metadata") {
            Ok(metadata) => arg::prop_cast(&metadata, key)
                .ok_or(Error::new_failed(&format!("Property '{}' not found in metadata", key)))
                .cloned(),
            
            Err(err) => Err(Error::new_failed(&format!("Failed to get metadata property '{}': {}", key, err)))
        }
    }

    pub fn sync_properties(&mut self) {
        macro_rules! set_metadata_property {
            ($type:ty, $key:expr, $field:ident) => {
                if let Ok(value) = self.get_metadata_property::<$type>($key) {
                    self.metadata.$field = Some(value);
                }
            }
        }

        let mut booleans = [
            "Shuffle",
            "CanGoNext",
            "CanGoPrevious",
            "CanPlay",
            "CanPause",
            "CanSeek",
            "CanControl"
        ];

        let mut f64s = [
            "Rate",
            "Volume",
            "MinimumRate",
            "MaximumRate"
        ];

        let mut i64s = [
            "Position"
        ];

        let mut strings = [
            "PlaybackStatus",
            "LoopStatus",
        ];

        for key in &mut booleans {
            let prop: bool = mpris_dbus::get_dbus_property::<bool>(self, key)
                .unwrap_or_else(|_| {
                    warn!(property = key, "Failed to get boolean property");
                    false
                });

            match *key {
                "Shuffle" => self.shuffle = prop,
                "CanGoNext" => self.can_go_next = prop,
                "CanGoPrevious" => self.can_go_previous = prop,
                "CanPlay" => self.can_play = prop,
                "CanPause" => self.can_pause = prop,
                "CanSeek" => self.can_seek = prop,
                "CanControl" => self.can_control = prop,
                _ => {}
            }
        }

        for key in &mut f64s {
            let prop: f64 = mpris_dbus::get_dbus_property::<f64>(self, key)
                .unwrap_or_else(|_| {
                    warn!(property = key, "Failed to get f64 property");
                    0.0
                });

            match *key {
                "Rate" => self.rate = prop,
                "Volume" => self.volume = prop,
                "MinimumRate" => self.minimum_rate = prop,
                "MaximumRate" => self.maximum_rate = prop,
                _ => {}
            }
        }

        for key in &mut i64s {
            let prop: i64 = mpris_dbus::get_dbus_property::<i64>(self, key)
                .unwrap_or_else(|_| {
                    warn!(property = key, "Failed to get i64 property");
                    0
                });

            if *key == "Position" { self.position = prop }
        }

        for key in &mut strings {
            let prop: String = mpris_dbus::get_dbus_property::<String>(self, key)
                .unwrap_or_else(|_| {
                    warn!(property = key, "Failed to get string property");
                    String::new()
                });

            match *key {
                "PlaybackStatus" => self.playback_status = PlaybackStatus::infer_from_string(&prop),
                "LoopStatus" => self.loop_status = LoopStatus::infer_from_string(&prop),
                _ => {}
            }
        }

        // Set metadata properties
        set_metadata_property!(String, "mpris:trackid", track_id);
        set_metadata_property!(i64, "mpris:length", length);
        set_metadata_property!(String, "mpris:artUrl", art_url);
        set_metadata_property!(String, "xesam:album", album);
        set_metadata_property!(Vec<String>, "xesam:artist", artist);
        set_metadata_property!(String, "xesam:contentCreated", content_created);
        set_metadata_property!(String, "xesam:title", title);
    }

    pub fn properties_changed(&mut self, msg: &Message) {
        let (_, props) = msg.get2::<String, arg::PropMap>();
        
        if let Some(props) = props {
            let mut booleans = [
                ("Shuffle", &mut self.shuffle),
                ("CanGoNext", &mut self.can_go_next),
                ("CanGoPrevious", &mut self.can_go_previous),
                ("CanPlay", &mut self.can_play),
                ("CanPause", &mut self.can_pause),
                ("CanSeek", &mut self.can_seek),
                ("CanControl", &mut self.can_control),
            ];

            let mut f64s = [
                ("Rate", &mut self.rate),
                ("Volume", &mut self.volume),
                ("MinimumRate", &mut self.minimum_rate),
                ("MaximumRate", &mut self.maximum_rate),
            ];

            if let Some(playback_status) = props.get("PlaybackStatus") {
                self.playback_status = PlaybackStatus::infer_from_string(playback_status.0.as_str().unwrap_or("Stopped"));
            }

            if let Some(loop_status) = props.get("LoopStatus") {
                self.loop_status = LoopStatus::infer_from_string(loop_status.0.as_str().unwrap_or("Stopped"));
            }

            if let Some(position) = props.get("Position") {
                if let Some(pos) = position.0.as_i64() {
                    self.position = pos;
                } else {
                    warn!(?position, "Failed to parse Position property");
                }
            }

            if let Some(metadata) = props.get("Metadata") {
                let kv = mpris_metadata::make_key_value_pairs(metadata);

                // Clear metadata before updating
                self.metadata = Metadata::default();

                for (key, value) in kv {
                    match key.as_str() {
                        "mpris:trackid" => self.metadata.track_id = Some(value.as_i64().unwrap_or_default().to_string()),
                        "mpris:length" => self.metadata.length = Some(value.as_i64().unwrap_or_default()),
                        "mpris:artUrl" => self.metadata.art_url = Some(value.as_str().unwrap_or_default().to_owned()),
                        "xesam:album" => self.metadata.album = Some(value.as_str().unwrap_or_default().to_owned()),
                        "xesam:artist" => self.metadata.artist = Some(mpris_metadata::as_str_vec(&value).unwrap_or_default()),
                        "xesam:contentCreated" => self.metadata.content_created = Some(value.as_str().unwrap_or_default().to_owned()),
                        "xesam:title" => self.metadata.title = Some(value.as_str().unwrap_or_default().to_owned()),
                        _ => {}
                    }
                }
            }

            for (key, flag) in &mut booleans {
                if let Some(value) = props.get(*key) {
                    if let Some(b) = value.0.as_i64() {
                        **flag = b != 0;
                    } else {
                        warn!(property = key, ?value, "Failed to parse boolean property");
                    }
                }
            }

            for (key, value) in &mut f64s {
                if let Some(prop) = props.get(*key) {
                    if let Some(v) = prop.0.as_f64() {
                        **value = v;
                    } else {
                        warn!(property = key, ?prop, "Failed to parse f64 property");
                    }
                }
            }
        } else {
            warn!(?msg, "PropertiesChanged message did not contain properties");
        }
    }

    pub fn seeked(&mut self, msg: &Message) {
        let nanos = msg.get1::<i64>().unwrap_or(0);

        self.position = nanos;
    }

    pub fn next(&mut self) -> Result<Message, Error> {
        if !self.can_go_next {
            return Err(Error::new_failed("Cannot go to next track, player does not support it"));
        }

        let result = mpris_dbus::run_dbus_method(self, "Next");

        // reset position if successful
        if result.is_ok() {
            self.position = 0;
        }

        result
    }

    pub fn previous(&mut self) -> Result<Message, Error> {
        if !self.can_go_previous {
            return Err(Error::new_failed("Cannot go to previous track, player does not support it"));
        }

        let result = mpris_dbus::run_dbus_method(self, "Previous");

        // reset position if successful
        if result.is_ok() {
            self.position = 0;
        }

        result
    }

    pub fn play(&self) -> Result<Message, Error> {
        if !self.can_play {
            return Err(Error::new_failed("Cannot play, player does not support it"));
        }

        mpris_dbus::run_dbus_method(self, "Play")
    }

    pub fn pause(&self) -> Result<Message, Error> {
        if !self.can_pause {
            return Err(Error::new_failed("Cannot pause, player does not support it"));
        }

        mpris_dbus::run_dbus_method(self, "Pause")
    }

    pub fn play_pause(&self) -> Result<Message, Error> {
        if !self.can_pause {
            return Err(Error::new_failed("Cannot play/pause, player does not support it"));
        }

        mpris_dbus::run_dbus_method(self, "PlayPause")
    }

    pub fn stop(&self) -> Result<Message, Error> {
        if !self.can_control {
            return Err(Error::new_failed("Cannot stop, player does not support it"));
        }

        mpris_dbus::run_dbus_method(self, "Stop")
    }

    pub fn seek(&self, position: i64) -> Result<Message, Error> {
        if !self.can_seek {
            return Err(Error::new_failed("Cannot seek, player does not support it"));
        }

        mpris_dbus::run_dbus_method_w_args::<i64>(self, "Seek", &[position])
    }

    pub fn set_loop_status(&self, status: LoopStatus) -> Result<(), Error> {
        if !self.can_control {
            return Err(Error::new_failed("Cannot set loop status, player does not support it"));
        }

        mpris_dbus::set_dbus_property(self, "LoopStatus", status.as_string())
    }

    pub fn set_shuffle(&self, shuffle: bool) -> Result<(), Error> {
        if !self.can_control {
            return Err(Error::new_failed("Cannot set shuffle, player does not support it"));
        }

        mpris_dbus::set_dbus_property(self, "Shuffle", shuffle)
    }

    pub fn adjust_volume(&self, delta: f64) -> Result<(), Error> {
        if !self.can_control {
            return Err(Error::new_failed("Cannot adjust volume, player does not support it"));
        }

        // 1.0 is only a sensible max, some players allow more than this
        // 1.5 is the true max limit for pulse/pipewire servers
        let new_volume = (self.volume + delta).clamp(0.0, 1.5);

        mpris_dbus::set_dbus_property(self, "Volume", new_volume)
    }

    pub fn get_and_update_position(&mut self) -> Result<i64, Error> {
        let position: i64 = mpris_dbus::get_dbus_property::<i64>(self, "Position")?;
        self.position = position;
        Ok(position)
    }
}