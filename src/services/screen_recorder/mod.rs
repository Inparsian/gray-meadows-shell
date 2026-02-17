use std::process::Stdio;
use std::sync::{OnceLock, RwLock};
use std::time::Duration;
use futures_signals::signal::{Mutable, SignalExt as _};
use tokio::process::Child;

use crate::config::{ScreenRecorderBitrateMode, read_config};
use crate::ipc;
use crate::services::notifications::client::NotificationBuilder;
use crate::utils::process::{self, send_signal};

pub static SCREEN_RECORDER: OnceLock<RwLock<ScreenRecorder>> = OnceLock::new();
const SIGINT_ATTEMPTS: u8 = 10;

#[derive(Clone, Debug)]
pub enum ScreenRecorderCaptureOption {
    Monitor(String, String),
    Portal,
}

impl ScreenRecorderCaptureOption {
    pub fn as_localized(&self) -> String {
        match self {
            ScreenRecorderCaptureOption::Monitor(name, res) => format!("Monitor {} ({})", name, res),
            ScreenRecorderCaptureOption::Portal => "Portal".to_owned(),
        }
    }
    
    pub fn as_config_option(&self) -> String {
        match self {
            ScreenRecorderCaptureOption::Monitor(name, _res) => name.clone(),
            ScreenRecorderCaptureOption::Portal => "portal".to_owned(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScreenRecorderAudioDevice {
    pub name: String,
    pub localized: String,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenRecorderState {
    Record,
    Replay,
    Waiting,
    #[default]
    Idle,
}

#[derive(Default, Debug)]
pub struct ScreenRecorder {
    pub state: Mutable<ScreenRecorderState>,
    pub old_state: Option<ScreenRecorderState>,
    pub process: Option<Child>,
    pub capture_options: Vec<ScreenRecorderCaptureOption>,
    pub audio_devices: Vec<ScreenRecorderAudioDevice>,
}

impl ScreenRecorder {
    pub fn new() -> Self {
        let me = Self::default();
        
        tokio::spawn(signal!(me.state, (new_state) {
            let (summary, body) = match new_state {
                ScreenRecorderState::Record => ("Recording Started", "Your recording has started."),
                ScreenRecorderState::Replay => ("Replay Started", "Your replay has started."),
                ScreenRecorderState::Waiting => ("What", "You should not see this message, if you do, please report it."),
                ScreenRecorderState::Idle => {
                    get_screen_recorder().read()
                        .map_or(None, |s| s.old_state)
                        .map_or(
                            ("What", "You should not see this message, if you do, please report it."),
                            |old_state| match old_state {
                                ScreenRecorderState::Record => ("Recording Stopped", "Your recording has stopped."),
                                ScreenRecorderState::Replay => ("Replay Stopped", "Your replay has stopped."),
                                _ => ("What", "You should not see this message, if you do, please report it.")
                            }
                        )
                },
            };
            
            if !matches!(new_state, ScreenRecorderState::Waiting)
                && (!matches!(new_state, ScreenRecorderState::Idle)
                    || get_screen_recorder().read().map_or(None, |s| s.old_state).is_some())
            {
                let _ = NotificationBuilder::new()
                    .app_name("gray-meadows-shell")
                    .summary(summary)
                    .body(body)
                    .send();
            }
        }));
        
        me
    }
    
    pub fn find_capture_option(&self, name: &str) -> Option<(usize, ScreenRecorderCaptureOption)> {
        self.capture_options.iter()
            .cloned()
            .enumerate()
            .find(|(_, option)| {
                if let ScreenRecorderCaptureOption::Monitor(n, _) = option {
                    n == name
                } else {
                    false
                }
            })
    }
    
    pub fn find_audio_device(&self, query: &str) -> Option<ScreenRecorderAudioDevice> {
        self.audio_devices.iter()
            .find(|&device| device.name == query || device.localized == query)
            .cloned()
    }
    
    pub fn set_state_and_old_state(&mut self, new_state: ScreenRecorderState) {
        self.old_state = Some(self.state.get());
        self.state.set(new_state);
    }
    
    pub fn start(&mut self, replay: bool) {
        if self.process.is_some() {
            return;
        }
        
        self.set_state_and_old_state(ScreenRecorderState::Waiting);
        
        let config = read_config().screen_recorder.clone();
        let window = self.find_capture_option(&config.capture_target)
            .map_or("portal".to_owned(), |t| t.1.as_config_option());
        
        let mut audio_sources = Vec::new();
        for target in config.audio_app_targets {
            audio_sources.push(format!("app:{}", target));
        }
        
        for target in config.audio_device_targets {
            if let Some(device) = self.find_audio_device(&target) {
                audio_sources.push(format!("device:{}", device.name));
            }
        }
        
        // Initial args
        let mut command = tokio::process::Command::new("gpu-screen-recorder");
        command
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .arg("-w").arg(&window)
            .arg("-c").arg(config.video_container.to_string())
            .arg("-f").arg(config.framerate.to_string())
            .arg("-cursor").arg(if config.record_cursor { "yes" } else { "no" })
            .arg("-a").arg(audio_sources.join("|"))
            .arg("-ac").arg(config.audio_codec.to_string())
            .arg("-k").arg(config.video_codec.to_string())
            .arg("-q").arg(if config.bitrate_mode == ScreenRecorderBitrateMode::ConstantBitrate {
                config.bitrate_kbps.to_string()
            } else {
                config.video_quality.to_string()
            })
            .arg("-bm").arg(config.bitrate_mode.to_string())
            .arg("-fm").arg(config.framerate_mode.to_string())
            .arg("-cr").arg(config.color_range.to_string())
            .arg("-o").arg(if replay {
                config.replay_output_directory
            } else {
                let today = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
                format!("{}/Video_{}.{}", config.recording_output_directory, today, config.video_container)
            });
            
        if replay {
            command.arg("-r").arg(config.replay_buffer_length_secs.to_string());
        }
            
        let child = command.spawn()
            .expect("failed to spawn gpu-screen-recorder");
        
        self.process = Some(child);
        self.state.set(if replay {
            ScreenRecorderState::Replay
        } else {
            ScreenRecorderState::Record
        });
    }
    
    pub fn stop(&mut self) {
        if let Some(mut process) = self.process.take() && let Some(pid) = process.id() {
            self.set_state_and_old_state(ScreenRecorderState::Waiting);
            
            tokio::spawn(async move {
                // It is possible that the process has not fully initialized yet, so
                // try_wait repeatedly until we get an exit status
                // This also ensures that any notifications regarding screen recorder
                // status updates won't show up on recording
                for attempts in 1..=SIGINT_ATTEMPTS {
                    send_signal(pid, "SIGINT");
                    
                    if !matches!(process.try_wait(), Ok(None)) {
                        break;
                    }
                    
                    if attempts >= SIGINT_ATTEMPTS {
                        warn!("Process is unresponsive, force-killing");
                        let _ = process.start_kill();
                    } else {
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                }
                
                if let Ok(screen_recorder) = get_screen_recorder().read() {
                    screen_recorder.state.set(ScreenRecorderState::Idle);
                }
            });
        }
    }
    
    pub fn toggle(&mut self, replay: bool) {
        match self.state.get() {
            ScreenRecorderState::Record if !replay => self.stop(),
            ScreenRecorderState::Replay if replay => self.stop(),
            ScreenRecorderState::Idle if self.process.is_none() => self.start(replay),
            _ => {},
        }
    }
    
    pub fn save_replay(&self) {
        if let Some(pid) = self.process.as_ref().and_then(|p| p.id()) {
            send_signal(pid, "SIGUSR1");
            
            // Send the notification with a slight delay to ensure it doesn't show
            // up on the replay
            tokio::spawn(async {
                tokio::time::sleep(Duration::from_millis(150)).await;
                let _ = NotificationBuilder::new()
                    .app_name("gray-meadows-shell")
                    .summary("Replay Saved")
                    .body("Your replay buffer has been saved.")
                    .send();
            });
        }
    }
}

pub fn get_screen_recorder<'a>() -> &'a RwLock<ScreenRecorder> {
    SCREEN_RECORDER.get_or_init(|| RwLock::new(ScreenRecorder::new()))
}

pub fn activate() {
    if !process::is_command_available("gpu-screen-recorder") {
        warn!("gpu-screen-recorder not found, skipping screen recorder service initialization");
        return;
    }
    
    let _ = SCREEN_RECORDER.set(RwLock::new(ScreenRecorder::new()));
    
    tokio::spawn(async {
        let capture_options = query_capture_options().await;
        let audio_devices = query_audio_devices().await;
        
        let mut recorder = get_screen_recorder().write().unwrap();
        recorder.capture_options = capture_options;
        recorder.audio_devices = audio_devices;
    });
    
    ipc::listen_for_messages_local(|message| {
        let message = message.as_str();
        if matches!(message, "screen_rec_start_recording"
            | "screen_rec_start_replay"
            | "screen_rec_toggle_recording"
            | "screen_rec_toggle_replay"
            | "screen_rec_save_replay"
            | "screen_rec_stop"
        ) && let Ok(mut screen_recorder) = get_screen_recorder().write() {
            match message {
                "screen_rec_start_recording" => screen_recorder.start(false),
                "screen_rec_start_replay" => screen_recorder.start(true),
                "screen_rec_toggle_recording" => screen_recorder.toggle(false),
                "screen_rec_toggle_replay" => screen_recorder.toggle(true),
                "screen_rec_save_replay" => screen_recorder.save_replay(),
                "screen_rec_stop" => screen_recorder.stop(),
                _ => unreachable!(),
            }
        }
    });
}

pub async fn query_capture_options() -> Vec<ScreenRecorderCaptureOption> {
    let result = tokio::process::Command::new("gpu-screen-recorder")
        .arg("--list-capture-options")
        .output()
        .await
        .map_err(|e| format!("Failed to list capture options: {}", e))
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter_map(|line| if line.trim() == "portal" {
                    Some(ScreenRecorderCaptureOption::Portal)
                } else {
                    let parts: Vec<&str> = line.split('|').collect();
                    
                    (parts.len() == 2)
                        .then(|| ScreenRecorderCaptureOption::Monitor(parts[0].to_owned(), parts[1].to_owned()))
                })
                .collect()
        });
    
    match result {
        Ok(options) => options,
        Err(err) => {
            error!(%err, "Failed to list capture options");
            Vec::new()
        }
    }
}

pub async fn query_audio_devices() -> Vec<ScreenRecorderAudioDevice> {
    let result = tokio::process::Command::new("gpu-screen-recorder")
        .arg("--list-audio-devices")
        .output()
        .await
        .map_err(|e| format!("Failed to list audio devices: {}", e))
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split('|').collect();
                    
                    (parts.len() == 2)
                        .then(|| ScreenRecorderAudioDevice {
                            name: parts[0].to_owned(),
                            localized: parts[1].to_owned(),
                        })
                })
                .collect()
        });
    
    match result {
        Ok(devices) => devices,
        Err(err) => {
            error!(%err, "Failed to list audio devices");
            Vec::new()
        }
    }
}

pub fn get_configured_capture_target() -> Option<(usize, ScreenRecorderCaptureOption)> {
    let capture_target = read_config().screen_recorder.capture_target.clone();
    
    get_screen_recorder()
        .read()
        .unwrap()
        .find_capture_option(&capture_target)
}