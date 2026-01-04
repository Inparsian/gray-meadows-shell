mod tools;
mod variables;
mod services;
pub mod types;
pub mod conversation;

use std::sync::{Arc, LazyLock, OnceLock, RwLock};

use crate::config::read_config;
use crate::sql::wrappers::aichats;
use crate::broadcast::BroadcastChannel;
use self::types::{
    AiSession,
    AiConversation, AiConversationItem, AiConversationItemPayload, AiConversationDelta,
};

pub static CHANNEL: OnceLock<BroadcastChannel<AiChannelMessage>> = OnceLock::new();
pub static SESSION: OnceLock<AiSession> = OnceLock::new();

static SERVICES: LazyLock<Vec<Box<dyn services::AiService>>> = LazyLock::new(|| vec![
    Box::new(services::openai::OpenAiService::default()),
]);

pub const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Debug, Clone)]
pub enum AiChannelMessage {
    StreamStart,
    StreamChunk(AiConversationDelta),
    StreamComplete(i64), // message ID
    ToolCall(String, String), // (tool name, arguments)
    CycleStarted,
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

fn write_item(item: &AiConversationItem) -> i64 {
    let Some(session) = SESSION.get() else {
        eprintln!("AI session not initialized");
        return 0;
    };

    let mut item = item.clone();
    item.conversation_id = current_conversation_id().unwrap_or(0);
    match aichats::add_item(&item) {
        Ok(id) => {
            item.id = id;
            session.items.write().unwrap().push(item);
            id
        },

        Err(err) => {
            eprintln!("Failed to save AI message to database: {}", err);
            0
        },
    }
}

pub fn trim_items(down_to_item_id: i64) {
    if let Some(session) = SESSION.get() {
        let conversation = session.conversation.read().unwrap();
        let Some(conversation) = &*conversation else {
            eprintln!("AI conversation not initialized");
            return;
        };

        if let Err(err) = aichats::trim_items(conversation.id, down_to_item_id) {
            eprintln!("Failed to trim AI chat items in database: {}", err);
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

pub fn activate() {
    let app_config = read_config();
    if !app_config.ai.enabled || app_config.ai.openai.api_key.is_empty() {
        return;
    }

    aichats::ensure_default_conversation().unwrap_or_else(|err| {
        eprintln!("Failed to ensure default AI chat conversation: {}", err);
    });

    let session = AiSession {
        conversation: Arc::new(RwLock::new(None)),
        items: Arc::new(RwLock::new(Vec::new())),
        currently_in_cycle: Arc::new(RwLock::new(false)),
        stop_cycle_flag: Arc::new(RwLock::new(false)),
    };

    let _ = SESSION.set(session);
    let _ = CHANNEL.set(BroadcastChannel::new(100));

    conversation::load_first_conversation();
}

pub async fn start_request_cycle() {
    let Some(session) = SESSION.get() else {
        eprintln!("AI session not initialized");
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
        eprintln!("AI channel not initialized");
        return;
    };

    channel.send(AiChannelMessage::CycleStarted).await;

    let config = read_config().clone();
    let service = SERVICES.iter()
        .find(|s| s.service_name() == config.ai.service)
        .unwrap_or(&SERVICES[0]);
    
    loop {
        let items = session.items.read().unwrap().clone();
        let stop_cycle_flag = session.stop_cycle_flag.clone();

        match service.make_stream_request(items, channel, stop_cycle_flag).await {
            Ok(result) => {
                for (index, item) in result.items.iter().enumerate() {
                    let id = write_item(item);
                    if index == 0 {
                        channel.send(AiChannelMessage::StreamComplete(id)).await;
                    }
                }

                // If this yielded any function calls, they must be processed
                let function_calls = result.items.iter()
                    .filter_map(|item| {
                        if let AiConversationItemPayload::FunctionCall {
                            call_id,
                            name,
                            arguments,
                            ..
                        } = &item.payload {
                            Some((call_id.clone(), name.clone(), arguments.clone()))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<(String, String, String)>>();

                let mut handles = Vec::new();
                for (call_id, name, arguments) in function_calls {
                    let handle = tokio::spawn({
                        let id = call_id.clone();
                        let name = name.clone();
                        let args = arguments.clone();
                        async move {
                            let result = tools::call_tool(&name, &args);
                            (id, result)
                        }
                    });

                    handles.push(handle);
                }

                let mut function_call_outputs = Vec::new();
                for handle in handles {
                    match handle.await {
                        Ok((call_id, output)) => {
                            function_call_outputs.push((call_id, output));
                        },

                        Err(e) => {
                            eprintln!("Failed to join tool call task: {:#?}", e);
                        }
                    }
                }

                for (call_id, output) in function_call_outputs {
                    let item = AiConversationItem {
                        id: 0,
                        conversation_id: current_conversation_id().unwrap_or(0),
                        payload: AiConversationItemPayload::FunctionCallOutput {
                            call_id: call_id.clone(),
                            output: output.to_string(),
                        },
                        timestamp: Some(chrono::Local::now().naive_local().to_string()),
                    };

                    write_item(&item);
                }

                // If more data should not be requested, break the cycle
                if !result.should_request_more {
                    break;
                }
            },

            Err(e) => {
                eprintln!("AI request failed: {:#?}", e);
                break;
            },
        }
    }

    channel.send(AiChannelMessage::CycleFinished).await;

    let mut currently_in_cycle = session.currently_in_cycle.write().unwrap();
    *currently_in_cycle = false;
}

pub fn send_user_message(message: &str) -> i64 {
    if SESSION.get().is_none() {
        eprintln!("AI session not initialized");
        return 0;
    }

    write_item(&AiConversationItem {
        id: 0,
        conversation_id: current_conversation_id().unwrap_or(0),
        payload: AiConversationItemPayload::Message {
            id: String::new(),
            role: "user".to_owned(),
            content: message.to_owned(),
        },
        timestamp: Some(chrono::Local::now().naive_local().to_string()),
    })
}