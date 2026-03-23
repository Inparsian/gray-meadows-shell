use serde::{Deserialize, Serialize};
use strum::EnumString;

#[derive(Debug, Clone, Serialize, Deserialize, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum OpenAiServiceTier {
    Flex,
    Priority,
    Default,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum OpenAiReasoningEffort {
    None,
    Minimal,
    Low,
    Medium,
    High,
    Xhigh,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum GeminiThinkingLevel {
    Low,
    High,
    Budget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum AiService {
    OpenAi,
    Gemini,
}