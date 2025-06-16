use dbus::Message;

#[derive(Debug, Clone, Copy)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
    Unknown
}

#[derive(Debug, Clone, Copy)]
pub struct MprisPlayer {
    pub bus: internment::Intern<String>,
    pub owner: internment::Intern<String>,
    pub position: i64, // this is in microseconds (1e-6 seconds)
    pub playback_status: PlaybackStatus,
    pub volume: f64
}

impl MprisPlayer {
    pub fn new(bus: String, owner: String) -> Self {
        MprisPlayer {
            bus: internment::Intern::new(bus),
            owner: internment::Intern::new(owner),
            position: 0,
            playback_status: PlaybackStatus::Unknown,
            volume: 0.5,
        }
    }

    pub fn properties_changed(&mut self, msg: &Message) {
        let (_, props) = msg.get2::<String, dbus::arg::PropMap>();

        if let Some(props) = props {
            if let Some(playback_status) = props.get("PlaybackStatus") {
                let deref = playback_status.0.as_str().unwrap_or("Unknown");

                self.playback_status = match deref {
                    "Playing" => PlaybackStatus::Playing,
                    "Paused" => PlaybackStatus::Paused,
                    "Stopped" => PlaybackStatus::Stopped,
                    _ => PlaybackStatus::Unknown,
                };
            }

            if let Some(volume) = props.get("Volume") {
                if let Some(vol) = volume.0.as_f64() {
                    self.volume = vol;
                } else {
                    eprintln!("Failed to parse Volume property: {:?}", volume);
                }
            }
            
            println!("{}::PropertiesChanged - {:?}", self.bus, msg);
        } else {
            eprintln!("PropertiesChanged message did not contain properties: {:?}", msg);
        }
    }

    pub fn seeked(&mut self, msg: &Message) {
        let nanos = msg.get1::<i64>().unwrap_or(0);

        self.position = nanos;

        println!("[{}] {}::Seeked", self.position, self.bus);
    }
}