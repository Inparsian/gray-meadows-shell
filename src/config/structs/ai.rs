use serde::{Deserialize, Serialize};

use super::deserialize_insensitive;
use super::super::enums::{
    OpenAiServiceTier,
    OpenAiReasoningEffort,
    GeminiThinkingLevel,
    AiService,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiFeatures {
    pub power_control: bool,
    pub mpris_control: bool,
    pub weather_info: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    pub api_key: String,
    pub model: String,
    #[serde(deserialize_with = "deserialize_insensitive")]
    pub service_tier: OpenAiServiceTier,
    #[serde(deserialize_with = "deserialize_insensitive")]
    pub reasoning_effort: OpenAiReasoningEffort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    pub api_key: String,
    pub model: String,
    pub thinking_budget: i64,
    #[serde(deserialize_with = "deserialize_insensitive")]
    pub thinking_level: GeminiThinkingLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub enabled: bool,
    #[serde(deserialize_with = "deserialize_insensitive")]
    pub service: AiService,
    pub prompt: String,
    pub user_message_timestamps: bool,
    pub assistant_name: Option<String>,
    pub assistant_icon_path: Option<String>,
    pub openai: OpenAiConfig,
    pub gemini: GeminiConfig,
    pub features: AiFeatures,
}