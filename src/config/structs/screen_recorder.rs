use serde::{Deserialize, Serialize};

use super::deserialize_insensitive;
use super::super::enums::{
    ScreenRecorderVideoContainer,
    ScreenRecorderVideoQuality,
    ScreenRecorderVideoCodec,
    ScreenRecorderAudioCodec,
    ScreenRecorderFramerateMode,
    ScreenRecorderBitrateMode,
    ScreenRecorderColorRange,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenRecorderConfig {
    // General
    // this is a string because displays can be specified as well.
    // this is validated at runtime instead of during config loading
    pub capture_target: String,
    pub replay_buffer_length_secs: u32,
    pub recording_output_directory: String,
    pub replay_output_directory: String,
    
    // Video
    #[serde(deserialize_with = "deserialize_insensitive")]
    pub video_container: ScreenRecorderVideoContainer,
    #[serde(deserialize_with = "deserialize_insensitive")]
    pub video_quality: ScreenRecorderVideoQuality,
    pub bitrate_kbps: u32,
    pub framerate: u32,
    #[serde(deserialize_with = "deserialize_insensitive")]
    pub video_codec: ScreenRecorderVideoCodec,
    #[serde(deserialize_with = "deserialize_insensitive")]
    pub framerate_mode: ScreenRecorderFramerateMode,
    #[serde(deserialize_with = "deserialize_insensitive")]
    pub bitrate_mode: ScreenRecorderBitrateMode,
    #[serde(deserialize_with = "deserialize_insensitive")]
    pub color_range: ScreenRecorderColorRange,
    pub record_cursor: bool,
    
    // Audio
    pub audio_app_targets: Vec<String>,
    pub audio_device_targets: Vec<String>,
    #[serde(deserialize_with = "deserialize_insensitive")]
    pub audio_codec: ScreenRecorderAudioCodec,
}