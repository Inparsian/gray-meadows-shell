use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};

use super::TIMESTAMP_FORMAT;

pub struct AiSession {
    pub conversation: Arc<RwLock<Option<AiConversation>>>,
    pub items: Arc<RwLock<Vec<AiConversationItem>>>,
    pub currently_in_cycle: Arc<RwLock<bool>>,
    pub stop_cycle_flag: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone)]
pub struct AiFunction {
    pub name: String,
    pub description: String,
    pub strict: bool,
    pub schema: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct AiConversation {
    pub id: i64,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AiConversationItemPayload {
    Message {
        id: String,
        role: String,
        content: String,
    },

    Reasoning {
        id: String,
        summary: String,
        encrypted_content: String,
    },

    FunctionCall {
        id: String,
        name: String,
        arguments: String,
        call_id: String,
    },

    FunctionCallOutput {
        call_id: String,
        output: String,
    },
}

#[derive(Debug, Clone)]
pub enum AiConversationDelta {
    Message(String),
    Reasoning(String),
}

#[derive(Debug, Clone)]
pub struct AiConversationItem {
    pub id: i64,
    pub conversation_id: i64,
    pub payload: AiConversationItemPayload,
    pub timestamp: Option<String>,
}

impl AiConversationItem {
    pub fn timestamp_or_now(&self) -> String {
        self.timestamp.clone().map_or_else(
            || chrono::Local::now().format(TIMESTAMP_FORMAT).to_string(), 
            |timestamp| chrono::NaiveDateTime::parse_from_str(&timestamp, "%Y-%m-%d %H:%M:%S")
                .map_or(timestamp, |dt| {
                    let utc = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc);
                    let local = utc.with_timezone(&chrono::Local);
                    local.format(TIMESTAMP_FORMAT).to_string()
                })
        )
    }

    fn format_with_timestamp(timestamp: &str, content: &str) -> String {
        format!(
            "[Sent on {}] {}",
            timestamp,
            content
        )
    }

    pub fn inject_timestamp_into_content(&mut self) {
        let timestamp = self.timestamp_or_now();

        if let AiConversationItemPayload::Message { content, .. } = &mut self.payload {
            let new_content = Self::format_with_timestamp(&timestamp, content);
            *content = new_content;
        }
    }
}