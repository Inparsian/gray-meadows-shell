mod tools;
mod sql;
mod variables;
pub mod conversation;

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, OnceLock, RwLock};
use async_openai::types::evals::InputTextContent;
use futures_lite::stream::StreamExt as _;
use async_openai::Client;
use async_openai::error::OpenAIError;
use async_openai::config::OpenAIConfig;
use async_openai::types::responses::{
    AssistantRole,
    CreateResponseArgs,
    FunctionCallOutput, FunctionCallOutputItemParam, FunctionToolCall, 
    InputContent, InputMessage, InputRole,
    Item, MessageItem,
    OutputContent, OutputItem, OutputMessage, OutputMessageContent, OutputStatus, OutputTextContent,
    ReasoningItem,
    ResponseStream, ResponseStreamEvent,
    ServiceTier,
    Summary, SummaryPart
};

use crate::config::read_config;
use crate::sql::wrappers::aichats::{self, SqlAiConversation};
use crate::broadcast::BroadcastChannel;

pub static CHANNEL: OnceLock<BroadcastChannel<AIChannelMessage>> = OnceLock::new();
pub static SESSION: OnceLock<AISession> = OnceLock::new();

const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Debug, Clone)]
pub enum AIChannelMessage {
    StreamStart,
    StreamChunk(String),
    StreamComplete(i64), // message ID
    ToolCall(String, String), // (tool name, arguments)
    CycleStarted,
    CycleFinished,
    ConversationLoaded(SqlAiConversation),
    ConversationTrimmed(i64, i64), // (conversation ID, down to message ID)
    ConversationAdded(SqlAiConversation),
    ConversationRenamed(i64, String), // (conversation ID, new title)
    ConversationDeleted(i64), // conversation ID
}

#[derive(Debug, Clone)]
pub struct AISessionItem {
    pub id: i64,
    pub timestamp: Option<String>,
    pub item: Item,
}

impl AISessionItem {
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

        if let Item::Message(MessageItem::Input(input_msg)) = &mut self.item
            && let InputContent::InputText(text_content) = &mut input_msg.content.first_mut().unwrap()
        {
            let new_content = Self::format_with_timestamp(&timestamp, &text_content.text);
            text_content.text = new_content;
        }
    }
}

pub struct AISession {
    pub client: Client<OpenAIConfig>,
    pub conversation: Arc<RwLock<Option<SqlAiConversation>>>,
    pub items: Arc<RwLock<Vec<AISessionItem>>>,
    pub currently_in_cycle: Arc<RwLock<bool>>,
    pub stop_cycle_flag: Arc<RwLock<bool>>,
}

pub fn is_currently_in_cycle() -> bool {
    SESSION.get().is_some_and(|session| *session.currently_in_cycle.read().unwrap())
}

pub fn current_conversation_id() -> Option<i64> {
    let session = SESSION.get()?;
    let conversation = session.conversation.read().unwrap();
    conversation.as_ref().map(|conv| conv.id)
}

fn make_client() -> Client<OpenAIConfig> {
    let app_config = read_config();
    let config = if app_config.ai.service == "gemini" {
        OpenAIConfig::new()
            .with_api_base("https://generativelanguage.googleapis.com/v1beta/openai")
            .with_api_key(app_config.ai.gemini.api_key.as_str())
    } else {
        OpenAIConfig::new()
            .with_api_key(app_config.ai.openai.api_key.as_str())
    };

    Client::with_config(config)
}

fn write_item(item: &Item) -> i64 {
    let Some(session) = SESSION.get() else {
        eprintln!("AI session not initialized");
        return 0;
    };

    let Some(conversation) = &*session.conversation.read().unwrap() else {
        eprintln!("AI conversation not initialized");
        return 0;
    };

    let Some(sql_item) = sql::item_to_sql_item(item, conversation.id) else {
        eprintln!("Failed to convert AI item to SQL item");
        return 0;
    };

    match aichats::add_item(&sql_item) {
        Ok(id) => {
            session.items.write().unwrap().push(AISessionItem {
                id,
                timestamp: sql_item.timestamp.clone(),
                item: item.clone(),
            });

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
            channel.spawn_send(AIChannelMessage::ConversationTrimmed(conversation.id, down_to_item_id));
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

    let session = AISession {
        client: make_client(),
        conversation: Arc::new(RwLock::new(None)),
        items: Arc::new(RwLock::new(Vec::new())),
        currently_in_cycle: Arc::new(RwLock::new(false)),
        stop_cycle_flag: Arc::new(RwLock::new(false)),
    };

    let _ = SESSION.set(session);
    let _ = CHANNEL.set(BroadcastChannel::new(100));

    conversation::load_first_conversation();
}

async fn create_stream(session: &AISession) -> Result<ResponseStream, OpenAIError> {
    let app_config = read_config().clone();
    let mut sorted_items = session.items.read().unwrap()
        .clone()
        .iter_mut()
        .map(|item| {
            if app_config.ai.user_message_timestamps {
                item.inject_timestamp_into_content();
            }
            item.item.clone()
        })
        .collect::<Vec<Item>>();

    sorted_items.insert(0, Item::Message(MessageItem::Input(InputMessage {
        role: InputRole::Developer,
        content: vec![InputContent::InputText(InputTextContent {
            text: variables::transform_variables(&app_config.ai.prompt),
        })],
        status: None,
    })));

    // FIXME gemini is currently broken after switch to responses API
    let request = if app_config.ai.service == "gemini" {
        CreateResponseArgs::default()
            .max_output_tokens(2048_u32)
            .stream(true)
            .model(app_config.ai.gemini.model.as_str())
            .tools(tools::get_tools()?)
            .input(sorted_items)
            .build()?
    } else {
        CreateResponseArgs::default()
            .max_output_tokens(2048_u32)
            .stream(true)
            .model(app_config.ai.openai.model.as_str())
            .service_tier(match app_config.ai.openai.service_tier.as_str() {
                "flex" => ServiceTier::Flex,
                "priority" => ServiceTier::Priority,
                _ => ServiceTier::Default,
            })
            .tools(tools::get_tools()?)
            .input(sorted_items)
            .build()?
    };

    session.client.responses().create_stream_byot(request).await
}

pub fn make_request() -> Pin<Box<dyn Future<Output = anyhow::Result<bool>> + 'static + Send>> {
    Box::pin(async move {
        let Some(channel) = CHANNEL.get() else {
            return Err(anyhow::anyhow!("AI request channel not initialized"));
        };

        let Some(session) = SESSION.get() else {
            return Err(anyhow::anyhow!("AI session not initialized"));
        };

        let mut should_request_again = true;
        let mut execution_handles = Vec::new();
        let mut items: HashMap<String, Item> = HashMap::new();
        let mut stream = create_stream(session).await?;

        channel.send(AIChannelMessage::StreamStart).await;

        while let Some(event) = stream.next().await {
            if *session.stop_cycle_flag.read().unwrap() {
                break;
            }

            match event? {
                ResponseStreamEvent::ResponseOutputItemAdded(event) => {
                    println!("Output item added: {:?}", event.item);

                    match event.item {
                        OutputItem::Message(output_message) => {
                            let id = output_message.id.clone();
                            items.insert(id.clone(), Item::Message(MessageItem::Output(OutputMessage {
                                id,
                                role: AssistantRole::Assistant,
                                content: vec![],
                                status: OutputStatus::InProgress,
                            })));
                        },

                        OutputItem::Reasoning(output_reasoning) => {
                            let id = output_reasoning.id.clone();
                            items.insert(id.clone(), Item::Reasoning(ReasoningItem {
                                id,
                                summary: vec![],
                                content: None,
                                encrypted_content: None,
                                status: Some(OutputStatus::InProgress),
                            }));
                        },

                        OutputItem::FunctionCall(output_function_call) => {
                            let id = output_function_call.id.clone().unwrap_or_default();
                            items.insert(id.clone(), Item::FunctionCall(FunctionToolCall {
                                id: Some(id),
                                name: output_function_call.name.clone(),
                                arguments: output_function_call.arguments.clone(),
                                call_id: output_function_call.call_id.clone(),
                                status: Some(OutputStatus::InProgress),
                            }));
                        },

                        _ => {},
                    }
                },

                ResponseStreamEvent::ResponseContentPartAdded(event) => {
                    println!("Content part added: {:?}", event.part);

                    let id = event.item_id.clone();
                    match event.part {
                        OutputContent::OutputText(content) => {
                            if let Some(Item::Message(MessageItem::Output(output_message))) = items.get_mut(&id) {
                                output_message.content.push(OutputMessageContent::OutputText(OutputTextContent {
                                    text: content.text.clone(),
                                    annotations: vec![],
                                    logprobs: None,
                                }));
                            }
                        },

                        OutputContent::ReasoningText(content) => {
                            if let Some(Item::Reasoning(reasoning_item)) = items.get_mut(&id) {
                                reasoning_item.summary.push(SummaryPart::SummaryText(Summary {
                                    text: content.text.clone(),
                                }));
                            }
                        },

                        _ => {},
                    }
                },

                ResponseStreamEvent::ResponseOutputTextDelta(event) => {
                    println!("Output text delta: {}", event.delta);

                    let id = event.item_id.clone();
                    match items.get_mut(&id) {
                        Some(Item::Message(MessageItem::Output(output_message))) => {
                            if let Some(OutputMessageContent::OutputText(last_content)) = output_message.content.last_mut() {
                                last_content.text.push_str(&event.delta);
                            } else {
                                output_message.content.push(OutputMessageContent::OutputText(OutputTextContent {
                                    text: event.delta.clone(),
                                    annotations: vec![],
                                    logprobs: None,
                                }));
                            }

                            channel.send(AIChannelMessage::StreamChunk(event.delta.clone())).await;
                        },

                        Some(Item::Reasoning(reasoning_item)) => {
                            if let Some(SummaryPart::SummaryText(last_summary)) = reasoning_item.summary.last_mut() {
                                last_summary.text.push_str(&event.delta);
                            } else {
                                reasoning_item.summary.push(SummaryPart::SummaryText(Summary {
                                    text: event.delta.clone(),
                                }));
                            }
                        },

                        _ => {},
                    }
                },

                ResponseStreamEvent::ResponseOutputTextDone(event) => {
                    println!("Output text done for item ID: {}", event.item_id);
                },

                ResponseStreamEvent::ResponseContentPartDone(event) => {
                    println!("Content part done: {:?}", event.part);
                },

                ResponseStreamEvent::ResponseOutputItemDone(event) => {
                    println!("Output item done: {:?}", event.item);

                    let id = match &event.item {
                        OutputItem::Message(msg) => msg.id.clone(),
                        OutputItem::Reasoning(reasoning) => reasoning.id.clone(),
                        OutputItem::FunctionCall(func_call) => func_call.id.clone().unwrap_or_default(),
                        _ => String::new(),
                    };

                    if let Some(item) = items.get_mut(&id) {
                        match item {
                            Item::Message(MessageItem::Output(output_message)) => {
                                output_message.status = OutputStatus::Completed;

                                output_message.content = match &event.item {
                                    OutputItem::Message(msg) => msg.content.clone(),
                                    _ => output_message.content.clone(),
                                };
                            },

                            Item::Reasoning(reasoning_item) => {
                                reasoning_item.status = Some(OutputStatus::Completed);

                                reasoning_item.encrypted_content = match &event.item {
                                    OutputItem::Reasoning(r) => r.encrypted_content.clone(),
                                    _ => reasoning_item.encrypted_content.clone(),
                                };

                                reasoning_item.summary = match &event.item {
                                    OutputItem::Reasoning(r) => r.summary.clone(),
                                    _ => reasoning_item.summary.clone(),
                                };
                            },

                            Item::FunctionCall(func_call) => {
                                func_call.status = Some(OutputStatus::Completed);

                                func_call.arguments = match &event.item {
                                    OutputItem::FunctionCall(fc) => fc.arguments.clone(),
                                    _ => func_call.arguments.clone(),
                                };

                                let handle = tokio::spawn({
                                    let id = func_call.call_id.clone();
                                    let name = func_call.name.clone();
                                    let args = func_call.arguments.clone();
                                    async move {
                                        let result = tools::call_tool(&name, &args);
                                        (id, result)
                                    }
                                });

                                // Do not request again if a power action is being performed
                                // Because users quite literally can not see AI responses if their
                                // system is powered off
                                if func_call.name == "perform_power_action" {
                                    should_request_again = false;
                                }

                                execution_handles.push(handle);
                                channel.send(AIChannelMessage::ToolCall(
                                    func_call.name.clone(),
                                    func_call.arguments.clone()
                                )).await;
                            },

                            _ => {},
                        }
                    }
                },

                _ => {},
            }
        }

        println!("Final items: {:#?}", items);

        if let Ok(mut stop_flag) = session.stop_cycle_flag.write() {
            *stop_flag = false;
        }

        for item in items.values() {
            write_item(item);
        }

        if !execution_handles.is_empty() {
            let mut tool_responses = Vec::new();
            for handle in execution_handles {
                let (tool_call_id, response) = handle.await?;
                tool_responses.push((tool_call_id, response));
            }

            for (tool_call_id, response) in tool_responses {
                write_item(&Item::FunctionCallOutput(FunctionCallOutputItemParam {
                    call_id: tool_call_id,
                    output: FunctionCallOutput::Text(response.to_string()),
                    id: None,
                    status: None,
                }));
            }

            // Go for another request after tool execution, in case the AI wants to say
            // something after tool execution or perform more tool calls
            Ok(should_request_again)
        } else {
            // We're done
            channel.send(AIChannelMessage::StreamComplete(0)).await;

            Ok(false)
        }
    })
}

pub async fn start_request_cycle() {
    if let Some(session) = SESSION.get() {
        let mut currently_in_cycle = session.currently_in_cycle.write().unwrap();
        if *currently_in_cycle {
            // Already in a request cycle
            return;
        }
        *currently_in_cycle = true;
    } else {
        eprintln!("AI session not initialized");
        return;
    }

    if let Some(channel) = CHANNEL.get() {
        channel.send(AIChannelMessage::CycleStarted).await;
    }

    loop {
        match make_request().await {
            Ok(should_request_again) if !should_request_again => break,
            Ok(_) => {},
            Err(e) => {
                eprintln!("AI request failed: {:#?}", e);
                break;
            }
        }
    }

    if let Some(channel) = CHANNEL.get() {
        channel.send(AIChannelMessage::CycleFinished).await;
    }

    if let Some(session) = SESSION.get() {
        let mut currently_in_cycle = session.currently_in_cycle.write().unwrap();
        *currently_in_cycle = false;
    }
}

pub fn send_user_message(message: &str) -> i64 {
    if SESSION.get().is_none() {
        eprintln!("AI session not initialized");
        return 0;
    }

    write_item(&Item::Message(MessageItem::Input(InputMessage {
        role: InputRole::User,
        content: vec![InputContent::InputText(InputTextContent {
            text: message.to_owned(),
        })],
        status: None,
    })))
}