use std::sync::{OnceLock, RwLock};
use tokio::process::Child;

use crate::config::read_config;
use crate::utils::process;

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

#[derive(Default, Debug)]
pub struct ScreenRecorder {
    #[allow(dead_code)] // TODO: Implement process management
    pub process: Option<Child>,
    pub capture_options: Vec<ScreenRecorderCaptureOption>,
}

impl ScreenRecorder {
    pub fn find_capture_option_by_monitor_name(&self, name: &str) -> Option<(usize, ScreenRecorderCaptureOption)> {
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
}

pub fn activate() {
    if !process::is_command_available("gpu-screen-recorder") {
        warn!("gpu-screen-recorder not found, skipping screen recorder service initialization");
        return;
    }
    
    let _ = SCREEN_RECORDER.set(RwLock::new(ScreenRecorder::default()));
    
    tokio::spawn(async {
        let capture_options = query_capture_options().await;
        let mut recorder = SCREEN_RECORDER
            .get_or_init(|| RwLock::new(ScreenRecorder::default()))
            .write()
            .unwrap();
        
        recorder.capture_options = capture_options;
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

pub fn get_configured_capture_target() -> Option<(usize, ScreenRecorderCaptureOption)> {
    let capture_target = read_config().screen_recorder.capture_target.clone();
    
    SCREEN_RECORDER
        .get_or_init(|| RwLock::new(ScreenRecorder::default()))
        .read()
        .unwrap()
        .find_capture_option_by_monitor_name(&capture_target)
}

#[allow(dead_code)] // TODO: Implement process management
pub fn is_running() -> bool {
    SCREEN_RECORDER
        .get_or_init(|| RwLock::new(ScreenRecorder::default()))
        .read()
        .unwrap()
        .process.is_some()
}
