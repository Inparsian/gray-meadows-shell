use async_openai::types::chat::{
    ChatCompletionMessageToolCall,
    ChatCompletionMessageToolCalls,
    ChatCompletionRequestAssistantMessage,
    ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessageContent,
    ChatCompletionRequestToolMessage,
    ChatCompletionRequestToolMessageContent,
    ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent,
    FunctionCall
};

use crate::sql::wrappers::aichats;

pub fn sql_message_to_chat_message(msg: &aichats::SqlAiConversationMessage) -> Option<ChatCompletionRequestMessage> {
    match msg.role.as_str() {
        "user" => Some(ChatCompletionRequestUserMessage::from(msg.content.as_str()).into()),

        "tool" => {
            let tool_call_id = msg.tool_calls.first()
                .map(|tc| tc.id.clone())
                .unwrap_or_default();

            Some(ChatCompletionRequestToolMessage {
                content: ChatCompletionRequestToolMessageContent::Text(msg.content.clone()),
                tool_call_id,
            }.into())
        },

        "assistant" => {
            let tool_calls: Vec<ChatCompletionMessageToolCalls> = msg.tool_calls.iter().map(|tc| {
                ChatCompletionMessageToolCalls::Function(
                    ChatCompletionMessageToolCall {
                        id: tc.id.clone(),
                        function: FunctionCall {
                            name: tc.function.clone(),
                            arguments: tc.arguments.clone(),
                        },
                    }
                )
            }).collect();

            Some(ChatCompletionRequestAssistantMessage {
                content: Some(ChatCompletionRequestAssistantMessageContent::from(msg.content.as_str())),
                tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
                ..Default::default()
            }.into())
        },
        
        _ => {
            eprintln!("Unknown message role: {}", msg.role);
            None
        },
    }
}

pub fn chat_message_to_sql_message(msg: &ChatCompletionRequestMessage, conversation_id: i64) -> aichats::SqlAiConversationMessage {
    let now = chrono::Utc::now().format(super::TIMESTAMP_FORMAT).to_string();
    match msg {
        ChatCompletionRequestMessage::System(system_msg) => aichats::SqlAiConversationMessage {
            id: 0,
            conversation_id,
            role: "system".to_owned(),
            content: match &system_msg.content {
                ChatCompletionRequestSystemMessageContent::Text(str) => str.clone(),
                _ => String::new(),
            },
            tool_calls: Vec::new(),
            timestamp: Some(now),
        },

        ChatCompletionRequestMessage::User(user_msg) => aichats::SqlAiConversationMessage {
            id: 0,
            conversation_id,
            role: "user".to_owned(),
            content: match &user_msg.content {
                ChatCompletionRequestUserMessageContent::Text(str) => str.clone(),
                _ => String::new(),
            },
            tool_calls: Vec::new(),
            timestamp: Some(now),
        },

        ChatCompletionRequestMessage::Tool(tool_msg) => aichats::SqlAiConversationMessage {
            id: 0,
            conversation_id,
            role: "tool".to_owned(),
            content: match &tool_msg.content {
                ChatCompletionRequestToolMessageContent::Text(str) => str.clone(),
                _ => String::new(),
            },
            tool_calls: vec![aichats::SqlAiConversationToolCall {
                id: tool_msg.tool_call_id.clone(),
                function: String::new(),
                arguments: String::new(),
            }],
            timestamp: Some(now),
        },

        ChatCompletionRequestMessage::Assistant(assistant_msg) => {
            let tool_calls = assistant_msg.tool_calls.as_ref()
                .map_or_else(Vec::new, |calls| {
                    calls.iter().filter_map(|call| {
                        match call {
                            ChatCompletionMessageToolCalls::Function(tool) => {
                                aichats::SqlAiConversationToolCall {
                                    id: tool.id.clone(),
                                    function: tool.function.name.clone(),
                                    arguments: tool.function.arguments.clone(),
                                }.into()
                            },

                            _ => None,
                        }
                    }).collect()
                });

            aichats::SqlAiConversationMessage {
                id: 0,
                conversation_id,
                role: "assistant".to_owned(),
                content: match &assistant_msg.content {
                    Some(ChatCompletionRequestAssistantMessageContent::Text(str)) => str.clone(),
                    _ => String::new(),
                },
                tool_calls,
                timestamp: Some(now),
            }
        },

        _ => panic!("Unknown ChatCompletionRequestMessage variant"),
    }
}