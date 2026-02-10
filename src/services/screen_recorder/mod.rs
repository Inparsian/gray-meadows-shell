use std::process::Stdio;
use std::sync::{OnceLock, RwLock};
use std::time::Duration;
use tokio::process::Child;

use crate::config::{ScreenRecorderBitrateMode, read_config};
use crate::utils::process::{self, send_signal};

pub static SCREEN_RECORDER: OnceLock<RwLock<ScreenRecorder>> = OnceLock::new();

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

#[derive(Default, Debug)]
pub struct ScreenRecorder {
    pub process: Option<Child>,
    pub capture_options: Vec<ScreenRecorderCaptureOption>,
    pub audio_devices: Vec<ScreenRecorderAudioDevice>,
}

impl ScreenRecorder {
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
    
    pub fn start(&mut self, replay: bool) {
        if self.process.is_some() {
            return;
        }
        
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
    }
    
    pub fn stop(&mut self) {
        if let Some(mut process) = self.process.take() && let Some(pid) = process.id() {
            tokio::spawn(async move {
                // it is possible that the process has not fully initialized yet
                // so try_wait repeatedly until we get an exit status
                while matches!(process.try_wait(), Ok(None)) {
                    send_signal(pid, "SIGINT");
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            });
        }
    }
    
    pub fn save_replay(&self) {
        if let Some(pid) = self.process.as_ref().and_then(|p| p.id()) {
            send_signal(pid, "SIGUSR1");
        }
    }
}

pub fn activate() {
    if !process::is_command_available("gpu-screen-recorder") {
        warn!("gpu-screen-recorder not found, skipping screen recorder service initialization");
        return;
    }
    
    let _ = SCREEN_RECORDER.set(RwLock::new(ScreenRecorder::default()));
    
    tokio::spawn(async {
        let capture_options = query_capture_options().await;
        let audio_devices = query_audio_devices().await;
        let mut recorder = SCREEN_RECORDER
            .get_or_init(|| RwLock::new(ScreenRecorder::default()))
            .write()
            .unwrap();
        
        recorder.capture_options = capture_options;
        recorder.audio_devices = audio_devices;
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
    
    SCREEN_RECORDER
        .get_or_init(|| RwLock::new(ScreenRecorder::default()))
        .read()
        .unwrap()
        .find_capture_option(&capture_target)
}