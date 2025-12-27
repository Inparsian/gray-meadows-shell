mod tools;
mod sql;

use std::pin::Pin;
use std::error::Error;
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
    CreateChatCompletionRequestArgs,
    FinishReason,
};

use crate::sql::wrappers::aichats::{self, SqlAiConversation};
use crate::{APP, broadcast::BroadcastChannel};

pub static CHANNEL: OnceLock<BroadcastChannel<AIChannelMessage>> = OnceLock::new();
pub static SESSION: OnceLock<AISession> = OnceLock::new();

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum AIChannelMessage {
    StreamStart,
    StreamChunk(String),
    StreamComplete,
    ToolCall(String, String), // (tool name, arguments)
    CycleStarted,
    CycleFinished,
    ConversationLoaded(SqlAiConversation),
}

pub struct AISession {
    pub client: Client<OpenAIConfig>,
    pub conversation: Arc<RwLock<Option<SqlAiConversation>>>,
    pub messages: Arc<RwLock<Vec<ChatCompletionRequestMessage>>>,
}

fn make_client() -> Client<OpenAIConfig> {
    let config = OpenAIConfig::new()
        .with_api_key(APP.config.ai.api_key.as_str());

    Client::with_config(config)
}

fn write_message(msg: &ChatCompletionRequestMessage) {
    let Some(session) = SESSION.get() else {
        eprintln!("AI session not initialized");
        return;
    };

    session.messages.write().unwrap().push(msg.clone());

    let Some(conversation) = &*session.conversation.read().unwrap() else {
        eprintln!("AI conversation not initialized");
        return;
    };

    let sql_message = sql::chat_message_to_sql_message(msg, conversation.id);
    if let Err(err) = aichats::add_message(&sql_message) {
        eprintln!("Failed to save AI message to database: {}", err);
    }
}

fn read_conversation(id: i64) -> Result<Vec<ChatCompletionRequestMessage>, Box<dyn Error>> {
    let sql_messages = aichats::get_messages(id)?;

    let chat_messages: Vec<ChatCompletionRequestMessage> = sql_messages
        .iter()
        .map(sql::sql_message_to_chat_message)
        .collect();

    Ok(chat_messages)
}

fn load_conversation(id: i64) {
    if let Some(session) = SESSION.get() {
        match read_conversation(id) {
            Ok(messages) => {
                let mut msgs = session.messages.write().unwrap();
                *msgs = messages;

                if let Some(channel) = CHANNEL.get() {
                    let conversation = session.conversation.read().unwrap();

                    channel.spawn_send(AIChannelMessage::ConversationLoaded(conversation.as_ref().unwrap().clone()));
                }
            },

            Err(err) => {
                eprintln!("Failed to load AI chat conversation from database: {}", err);
            }
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

    let conversation = match aichats::get_conversation(1) {
        Ok(conv) => conv,
        Err(err) => {
            eprintln!("Failed to load default AI chat conversation: {}", err);
            return;
        }
    };

    let session = AISession {
        client: make_client(),
        conversation: Arc::new(RwLock::new(Some(conversation))),
        messages: Arc::new(RwLock::new(vec![
            ChatCompletionRequestSystemMessage::from(APP.config.ai.prompt.as_str()).into()
        ])),
    };

    let _ = SESSION.set(session);
    let _ = CHANNEL.set(BroadcastChannel::new(100));

    if let Some(session) = SESSION.get() {
        let Some(conversation) = &*session.conversation.read().unwrap() else {
            eprintln!("AI conversation not initialized");
            return;
        };

        load_conversation(conversation.id);
    }
}

pub fn make_request() -> Pin<Box<dyn Future<Output = anyhow::Result<bool>> + 'static + Send>> {
    Box::pin(async move {
        let Some(channel) = CHANNEL.get() else {
            return Err(anyhow::anyhow!("AI request channel not initialized"));
        };

        let Some(session) = SESSION.get() else {
            return Err(anyhow::anyhow!("AI session not initialized"));
        };

        let request = CreateChatCompletionRequestArgs::default()
            .max_completion_tokens(2048_u32)
            .stream(true)
            .model(APP.config.ai.model.as_str())
            .messages(session.messages.read().unwrap().clone())
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
            
            channel.send(AIChannelMessage::StreamComplete).await;
            write_message(&message);

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
            channel.send(AIChannelMessage::StreamComplete).await;
            write_message(&message);

            Ok(false)
        }
    })
}

pub async fn start_request_cycle() {
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
}

pub fn send_user_message(message: &str) {
    if SESSION.get().is_none() {
        eprintln!("AI session not initialized");
        return;
    }

    write_message(&ChatCompletionRequestUserMessage::from(message).into());
}