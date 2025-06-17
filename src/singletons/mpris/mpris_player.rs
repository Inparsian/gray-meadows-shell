use dbus::Message;
use internment::Intern;

#[derive(Debug, Clone, Copy)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped
}

#[derive(Debug, Clone, Copy)]
pub enum LoopStatus {
    None,
    Track,
    Playlist
}

#[derive(Debug, Clone, Copy)]
pub struct MprisPlayer {
    pub bus: Intern<String>,
    pub owner: Intern<String>,
    pub playback_status: PlaybackStatus,
    pub loop_status: LoopStatus,
    pub rate: f64,
    pub shuffle: bool,
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
        let player = MprisPlayer {
            bus: Intern::new(bus),
            owner: Intern::new(owner),
            playback_status: PlaybackStatus::Stopped,
            loop_status: LoopStatus::None,
            rate: 1.0,
            shuffle: false,
            volume: 1.0,
            position: 0,
            minimum_rate: 0.5,
            maximum_rate: 2.0,
            can_go_next: false,
            can_go_previous: false,
            can_play: false,
            can_pause: false,
            can_seek: false,
            can_control: false
        };

        player
    }

    pub fn properties_changed(&mut self, msg: &Message) {
        let (_, props) = msg.get2::<String, dbus::arg::PropMap>();

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
                let deref = playback_status.0.as_str().unwrap_or("Unknown");

                self.playback_status = match deref {
                    "Playing" => PlaybackStatus::Playing,
                    "Paused" => PlaybackStatus::Paused,
                    _ => PlaybackStatus::Stopped,
                };
            }

            if let Some(loop_status) = props.get("LoopStatus") {
                let deref = loop_status.0.as_str().unwrap_or("Unknown");

                self.loop_status = match deref {
                    "Track" => LoopStatus::Track,
                    "Playlist" => LoopStatus::Playlist,
                    _ => LoopStatus::None,
                };
            }

            if let Some(position) = props.get("Position") {
                if let Some(pos) = position.0.as_i64() {
                    self.position = pos;
                } else {
                    eprintln!("Failed to parse Position property: {:?}", position);
                }
            }

            for (key, flag) in booleans.iter_mut() {
                if let Some(value) = props.get(*key) {
                    if let Some(b) = value.0.as_i64() {
                        **flag = b != 0;
                    } else {
                        eprintln!("Failed to parse {} property: {:?}", key, value);
                    }
                }
            }

            for (key, value) in f64s.iter_mut() {
                if let Some(prop) = props.get(*key) {
                    if let Some(v) = prop.0.as_f64() {
                        **value = v;
                    } else {
                        eprintln!("Failed to parse {} property: {:?}", key, prop);
                    }
                }
            }

            println!("{}::PropertiesChanged - playing:{} loop:{} rate:{} shuffle:{} vol:{} pos:{} min_rate:{} max_rate:{} can_go_next:{} can_go_previous:{} can_play:{} can_pause:{} can_seek:{} can_control:{}",
                self.bus, self.playback_status as u8, self.loop_status as u8,
                self.rate, self.shuffle, self.volume, self.position, self.minimum_rate,
                self.maximum_rate, self.can_go_next, self.can_go_previous,
                self.can_play, self.can_pause, self.can_seek, self.can_control);
        } else {
            eprintln!("PropertiesChanged message did not contain properties: {:?}", msg);
        }
    }

    pub fn seeked(&mut self, msg: &Message) {
        let nanos = msg.get1::<i64>().unwrap_or(0);

        self.position = nanos;

        println!("{}::Seeked - {}", self.bus, self.position);
    }
}