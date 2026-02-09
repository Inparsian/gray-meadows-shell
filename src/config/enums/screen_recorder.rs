use serde::{Deserialize, Serialize};
use strum::EnumString;

#[derive(Debug, Clone, Serialize, Deserialize, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum ScreenRecorderVideoContainer {
    #[strum(to_string = "mp4")]
    Mp4,
    #[strum(to_string = "flv")]
    Flv,
    #[strum(to_string = "mkv")]
    Mkv,
    #[strum(to_string = "mov")]
    Mov,
    #[strum(to_string = "ts")]
    Ts,
    #[strum(to_string = "m3u8")]
    M3U8,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum ScreenRecorderVideoQuality {
    #[strum(to_string = "medium")]
    Medium,
    #[strum(to_string = "high")]
    High,
    #[strum(to_string = "very_high", serialize = "veryhigh")]
    VeryHigh,
    #[strum(to_string = "ultra")]
    Ultra,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum ScreenRecorderVideoCodec {
    #[strum(to_string = "auto")]
    Auto,
    #[strum(to_string = "h264")]
    H264,
    #[strum(to_string = "hevc")]
    Hevc,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum ScreenRecorderAudioCodec {
    #[strum(to_string = "auto")]
    Auto,
    #[strum(to_string = "opus")]
    Opus,
    #[strum(to_string = "flac")]
    Flac,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum ScreenRecorderFramerateMode {
    #[strum(to_string = "cfr", serialize = "constant")]
    Constant,
    #[strum(to_string = "vfr", serialize = "variable")]
    Variable,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum ScreenRecorderBitrateMode {
    #[strum(to_string = "qp", serialize = "constantquality", serialize = "constant_quality")]
    ConstantQuality,
    #[strum(to_string = "vbr", serialize = "variablebitrate", serialize = "variable_bitrate")]
    VariableBitrate,
    #[strum(to_string = "cbr", serialize = "constantbitrate", serialize = "constant_bitrate")]
    ConstantBitrate,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum ScreenRecorderColorRange {
    #[strum(to_string = "limited")]
    Limited,
    #[strum(to_string = "full")]
    Full,
}