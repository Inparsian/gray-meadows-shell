mod tools;
mod variables;
mod services;
pub mod images;
pub mod types;
pub mod conversation;

use std::sync::{Arc, LazyLock, OnceLock, RwLock};

use crate::config::read_config;
use crate::sql::wrappers::aichats;
use crate::utils::broadcast::BroadcastChannel;
use self::types::{
    AiSession,
    AiConversation, AiConversationItem, AiConversationItemPayload, AiConversationDelta,
};

pub static CHANNEL: OnceLock<BroadcastChannel<AiChannelMessage>> = OnceLock::new();
pub static SESSION: OnceLock<AiSession> = OnceLock::new();

static SERVICES: LazyLock<Vec<Box<dyn services::AiService>>> = LazyLock::new(|| vec![
    Box::new(services::openai::OpenAiService::default()),
    Box::new(services::gemini::GeminiService {}),
]);

pub const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Debug, Clone)]
pub enum AiChannelMessage {
    StreamStart,
    StreamChunk(AiConversationDelta),
    StreamComplete(i64), // message ID
    StreamReasoningSummaryPartAdded,
    ToolCall(String, String), // (tool name, arguments)
    CycleStarted,
    CycleFailed,
    CycleFinished,
    ConversationLoaded(AiConversation),
    ConversationTrimmed(i64, i64), // (conversation ID, down to message ID)
    ConversationAdded(AiConversation),
    ConversationRenamed(i64, String), // (conversation ID, new title)
    ConversationDeleted(i64), // conversation ID
}

pub fn is_currently_in_cycle() -> bool {
    SESSION.get().is_some_and(|session| *session.currently_in_cycle.read().unwrap())
}

pub fn current_conversation_id() -> Option<i64> {
    let session = SESSION.get()?;
    let conversation = session.conversation.read().unwrap();
    conversation.as_ref().map(|conv| conv.id)
}

async fn write_item_payload(payload: AiConversationItemPayload) -> i64 {
    let Some(session) = SESSION.get() else {
        warn!("AI session not initialized");
        return 0;
    };

    let mut item = AiConversationItem {
        id: 0,
        conversation_id: current_conversation_id().unwrap_or(0),
        payload,
        timestamp: Some(chrono::Local::now().naive_local().to_string()),
    };

    match aichats::add_item(&item).await {
        Ok(id) => {
            item.id = id;
            session.items.write().unwrap().push(item);
            id
        },

        Err(err) => {
            error!(%err, "Failed to save AI message to database");
            0
        },
    }
}

pub async fn trim_items(down_to_item_id: i64) {
    if let Some(session) = SESSION.get() {
        let Some(conversation) = session.conversation.read().unwrap().clone() else {
            warn!("AI conversation not initialized");
            return;
        };

        if let Err(err) = aichats::trim_items(conversation.id, down_to_item_id).await {
            error!(%err, "Failed to trim AI chat items in database");
            return;
        }

        let mut write = session.items.write().unwrap();
        let indices = write.iter()
            .filter_map(|item| (item.id >= down_to_item_id).then_some(item.id))
            .collect::<Vec<i64>>();
        write.retain(|item| !indices.contains(&item.id));

        if let Some(channel) = CHANNEL.get() {
            channel.spawn_send(AiChannelMessage::ConversationTrimmed(conversation.id, down_to_item_id));
        }
    }
}

pub async fn activate() {
    let app_config = read_config().clone();
    if !app_config.ai.enabled || (app_config.ai.openai.api_key.is_empty() && app_config.ai.gemini.api_key.is_empty()) {
        return;
    }

    aichats::ensure_default_conversation().await.unwrap_or_else(|err| {
        error!(%err, "Failed to ensure default AI chat conversation");
    });

    let session = AiSession {
        conversation: Arc::new(RwLock::new(None)),
        items: Arc::new(RwLock::new(Vec::new())),
        currently_in_cycle: Arc::new(RwLock::new(false)),
        stop_cycle_flag: Arc::new(RwLock::new(false)),
    };

    let _ = SESSION.set(session);
    let _ = CHANNEL.set(BroadcastChannel::new(100));

    if let Ok(Some(id)) = aichats::get_state_conversation_id().await {
        conversation::load_conversation(id).await;
    } else {
        conversation::load_first_conversation().await;
    }

    images::collect_garbage().await;
}

pub async fn start_request_cycle() {
    let Some(session) = SESSION.get() else {
        warn!("AI session not initialized");
        return;
    };

    {
        let mut currently_in_cycle = session.currently_in_cycle.write().unwrap();
        if *currently_in_cycle {
            // Already in a request cycle
            return;
        }
        *currently_in_cycle = true;
    }

    let Some(channel) = CHANNEL.get() else {
        warn!("AI channel not initialized");
        return;
    };

    channel.send(AiChannelMessage::CycleStarted).await;

    let config = read_config().clone();
    let service = SERVICES.iter()
        .find(|s| s.service() == config.ai.service)
        .unwrap_or(&SERVICES[0]);
    
    let mut failed = false;
    loop {
        let items = session.items.read().unwrap()
            .clone()
            .iter_mut()
            .map(|item| {
                if config.ai.user_message_timestamps {
                    item.inject_timestamp_into_content();
                }
                item.clone()
            })
            .collect::<Vec<AiConversationItem>>();

        let stop_cycle_flag = session.stop_cycle_flag.clone();
        match service.make_stream_request(items, channel, stop_cycle_flag).await {
            Ok(result) => {
                for (index, item) in result.items.iter().enumerate() {
                    let id = write_item_payload(item.clone()).await;
                    if index == 0 {
                        channel.send(AiChannelMessage::StreamComplete(id)).await;
                    }
                }

                // If this yielded any function calls, they must be processed
                let handles = result.items.iter().filter_map(|payload| {
                    if let AiConversationItemPayload::FunctionCall { call_id, name, arguments, .. } = &payload {
                        let id = call_id.clone();
                        let name = name.clone();
                        let args = arguments.clone();
                        Some(tokio::spawn(async move {
                            let result = tools::call_tool(&name, &args);
                            (id, name, result)
                        }))
                    } else {
                        None
                    }
                }).collect::<Vec<_>>();

                let function_call_outputs = futures::future::join_all(handles).await.into_iter()
                    .filter_map(|res| match res {
                        Ok((call_id, name, output)) => Some(AiConversationItemPayload::FunctionCallOutput {
                            call_id,
                            output: output.to_string(),
                            name: Some(name),
                        }),
                        Err(e) => {
                            error!(?e, "Failed to join tool call task");
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                for payload in function_call_outputs {
                    write_item_payload(payload).await;
                }

                // If more data should not be requested, break the cycle
                if !result.should_request_more {
                    break;
                }
            },

            Err(e) => {
                error!(?e, "AI request failed");
                failed = true;
                break;
            },
        }
    }

    if failed {
        channel.send(AiChannelMessage::CycleFailed).await;
    } else {
        channel.send(AiChannelMessage::CycleFinished).await;
    }

    let mut currently_in_cycle = session.currently_in_cycle.write().unwrap();
    *currently_in_cycle = false;
}

pub async fn send_user_message(message: &str) -> i64 {
    if SESSION.get().is_none() {
        warn!("AI session not initialized");
        return 0;
    }

    write_item_payload(AiConversationItemPayload::Message {
        id: String::new(),
        role: "user".to_owned(),
        content: message.to_owned(),
        thought_signature: None,
    }).await
}

pub async fn send_user_image(uuid: &str) -> i64 {
    if SESSION.get().is_none() {
        warn!("AI session not initialized");
        return 0;
    }

    write_item_payload(AiConversationItemPayload::Image {
        uuid: uuid.to_owned(),
    }).await
}