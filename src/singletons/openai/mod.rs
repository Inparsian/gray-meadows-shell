mod tools;
mod sql;
mod variables;
pub mod conversation;

use std::pin::Pin;
use std::sync::{Arc, OnceLock, RwLock};
use futures_lite::stream::StreamExt as _;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::{
    ChatCompletionMessageToolCall,
    ChatCompletionMessageToolCalls,
    ChatCompletionRequestAssistantMessage,
    ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessage,
    ChatCompletionRequestToolMessage,
    ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent,
    CreateChatCompletionRequestArgs,
    FinishReason,
    ServiceTier,
};

use crate::sql::wrappers::aichats::{self, SqlAiConversation};
use crate::{APP, broadcast::BroadcastChannel};

pub static CHANNEL: OnceLock<BroadcastChannel<AIChannelMessage>> = OnceLock::new();
pub static SESSION: OnceLock<AISession> = OnceLock::new();

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
pub struct AISessionMessage {
    pub id: i64,
    pub timestamp: Option<String>,
    pub message: ChatCompletionRequestMessage,
}

impl AISessionMessage {
    pub fn timestamp_or_now(&self) -> String {
        self.timestamp.clone().unwrap_or_else(|| {
            chrono::Local::now().format("%A, %B %d, %Y at %I:%M %p %Z").to_string()
        })
    }

    fn format_with_timestamp(timestamp: &str, content: &str) -> String {
        format!(
            "[Sent at {}] {}",
            timestamp,
            content
        )
    }

    pub fn inject_timestamp_into_content(&mut self) {
        let timestamp = self.timestamp_or_now();
        if let ChatCompletionRequestMessage::User(user_msg) = &mut self.message
            && let ChatCompletionRequestUserMessageContent::Text(content) = &user_msg.content
        {
            let new_content = Self::format_with_timestamp(&timestamp, content);
            user_msg.content = ChatCompletionRequestUserMessageContent::Text(new_content);
        }
    }
}

pub struct AISession {
    pub client: Client<OpenAIConfig>,
    pub conversation: Arc<RwLock<Option<SqlAiConversation>>>,
    pub messages: Arc<RwLock<Vec<AISessionMessage>>>,
    pub currently_in_cycle: Arc<RwLock<bool>>,
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
    let config = OpenAIConfig::new()
        .with_api_key(APP.config.ai.api_key.as_str());

    Client::with_config(config)
}

fn write_message(msg: &ChatCompletionRequestMessage) -> i64 {
    let Some(session) = SESSION.get() else {
        eprintln!("AI session not initialized");
        return 0;
    };

    let Some(conversation) = &*session.conversation.read().unwrap() else {
        eprintln!("AI conversation not initialized");
        return 0;
    };

    let sql_message = sql::chat_message_to_sql_message(msg, conversation.id);
    match aichats::add_message(&sql_message) {
        Ok(id) => {
            session.messages.write().unwrap().push(AISessionMessage {
                id,
                timestamp: sql_message.timestamp.clone(),
                message: msg.clone(),
            });

            id
        },

        Err(err) => {
            eprintln!("Failed to save AI message to database: {}", err);
            0
        },
    }
}

pub fn trim_messages(down_to_message_id: i64) {
    if let Some(session) = SESSION.get() {
        let conversation = session.conversation.read().unwrap();
        let Some(conversation) = &*conversation else {
            eprintln!("AI conversation not initialized");
            return;
        };

        if let Err(err) = aichats::trim_messages(conversation.id, down_to_message_id) {
            eprintln!("Failed to trim AI chat messages in database: {}", err);
            return;
        }

        let mut write = session.messages.write().unwrap();
        let indices = write.iter()
            .filter_map(|msg| (msg.id >= down_to_message_id).then_some(msg.id))
            .collect::<Vec<i64>>();
        write.retain(|msg| !indices.contains(&msg.id));

        if let Some(channel) = CHANNEL.get() {
            channel.spawn_send(AIChannelMessage::ConversationTrimmed(conversation.id, down_to_message_id));
        }
    }
}

pub fn activate() {
    if !APP.config.ai.enabled || APP.config.ai.api_key.is_empty() {
        return;
    }

    aichats::ensure_default_conversation().unwrap_or_else(|err| {
        eprintln!("Failed to ensure default AI chat conversation: {}", err);
    });

    let session = AISession {
        client: make_client(),
        conversation: Arc::new(RwLock::new(None)),
        messages: Arc::new(RwLock::new(Vec::new())),
        currently_in_cycle: Arc::new(RwLock::new(false)),
    };

    let _ = SESSION.set(session);
    let _ = CHANNEL.set(BroadcastChannel::new(100));

    conversation::load_first_conversation();
}

pub fn make_request() -> Pin<Box<dyn Future<Output = anyhow::Result<bool>> + 'static + Send>> {
    Box::pin(async move {
        let Some(channel) = CHANNEL.get() else {
            return Err(anyhow::anyhow!("AI request channel not initialized"));
        };

        let Some(session) = SESSION.get() else {
            return Err(anyhow::anyhow!("AI session not initialized"));
        };

        let mut sorted_messages = session.messages.read().unwrap()
            .clone()
            .iter_mut()
            .map(|msg| {
                msg.inject_timestamp_into_content();
                msg.message.clone()
            })
            .collect::<Vec<ChatCompletionRequestMessage>>();

        sorted_messages.insert(0, ChatCompletionRequestSystemMessage::from(
            variables::transform_variables(APP.config.ai.prompt.as_str())).into()
        );

        let request = CreateChatCompletionRequestArgs::default()
            .max_completion_tokens(2048_u32)
            .stream(true)
            .model(APP.config.ai.model.as_str())
            .messages(sorted_messages)
            .service_tier(match APP.config.ai.service_tier.as_str() {
                "flex" => ServiceTier::Flex,
                "priority" => ServiceTier::Priority,
                _ => ServiceTier::Default,
            })
            .tools(tools::get_tools()?)
            .build()?;

        let mut should_request_again = true;
        let mut stream = session.client.chat().create_stream(request).await?;
        let mut tool_calls = Vec::new();
        let mut content_chunks = Vec::new();
        let mut execution_handles = Vec::new();

        channel.send(AIChannelMessage::StreamStart).await;

        while let Some(result) = stream.next().await {
            let response = result?;

            for choice in response.choices {
                if let Some(content) = &choice.delta.content {
                    channel.send(AIChannelMessage::StreamChunk(content.clone())).await;
                    content_chunks.push(content.clone());
                }

                if let Some(tool_call_chunks) = choice.delta.tool_calls {
                    for chunk in tool_call_chunks {
                        let index = chunk.index as usize;

                        while tool_calls.len() <= index {
                            tool_calls.push(ChatCompletionMessageToolCall {
                                id: String::new(),
                                function: Default::default(),
                            });
                        }

                        let tool_call = &mut tool_calls[index];
                        if let Some(id) = chunk.id {
                            tool_call.id = id;
                        }

                        if let Some(function_chunk) = chunk.function {
                            if let Some(name) = function_chunk.name {
                                tool_call.function.name = name;

                                if tool_call.function.name == "perform_power_action" {
                                    // If a power action is being performed, we won't request again
                                    should_request_again = false;
                                }
                            }

                            if let Some(arguments) = function_chunk.arguments {
                                tool_call.function.arguments.push_str(&arguments);
                            }
                        }
                    }
                }

                if choice.finish_reason == Some(FinishReason::ToolCalls) {
                    for tool_call in &tool_calls {
                        let handle = tokio::spawn({
                            let id = tool_call.id.clone();
                            let name = tool_call.function.name.clone();
                            let args = tool_call.function.arguments.clone();
                            async move {
                                let result = tools::call_tool(&name, &args);
                                (id, result)
                            }
                        });

                        execution_handles.push(handle);
                        channel.send(AIChannelMessage::ToolCall(
                            tool_call.function.name.clone(),
                            tool_call.function.arguments.clone(),
                        )).await;
                    }
                }
            }
        };

        let joined_content: String = content_chunks.join("");
        let content = if joined_content.is_empty() {
            None
        } else {
            Some(ChatCompletionRequestAssistantMessageContent::from(joined_content.clone()))
        };

        if !execution_handles.is_empty() {
            let mut tool_responses = Vec::new();
            for handle in execution_handles {
                let (tool_call_id, response) = handle.await?;
                tool_responses.push((tool_call_id, response));
            }

            let assistant_tool_calls: Vec<ChatCompletionMessageToolCalls> = tool_calls
                .iter()
                .map(|tc| tc.clone().into())
                .collect();

            let message: ChatCompletionRequestMessage = ChatCompletionRequestAssistantMessage {
                content,
                tool_calls: Some(assistant_tool_calls),
                ..Default::default()
            }.into();
            
            let id = write_message(&message);
            channel.send(AIChannelMessage::StreamComplete(id)).await;

            for (tool_call_id, response) in tool_responses {
                write_message(&ChatCompletionRequestToolMessage {
                    content: response.to_string().into(),
                    tool_call_id,
                }.into());
            }

            // Go for another request after tool execution, in case the AI wants to say
            // something after tool execution or perform more tool calls
            Ok(should_request_again)
        } else {
            let message: ChatCompletionRequestMessage = ChatCompletionRequestAssistantMessage {
                content,
                ..Default::default()
            }.into();

            // We're done
            let id = write_message(&message);
            channel.send(AIChannelMessage::StreamComplete(id)).await;

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
                eprintln!("AI request failed: {}", e);
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

    write_message(&ChatCompletionRequestUserMessage::from(message).into())
}