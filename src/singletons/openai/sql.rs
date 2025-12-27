use async_openai::types::chat::{
    ChatCompletionMessageToolCall,
    ChatCompletionMessageToolCalls,
    ChatCompletionRequestAssistantMessage,
    ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent,
    ChatCompletionRequestToolMessage,
    ChatCompletionRequestToolMessageContent,
    ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent,
    FunctionCall
};

use crate::sql::wrappers::aichats;

pub fn sql_message_to_chat_message(msg: &aichats::SqlAiConversationMessage) -> ChatCompletionRequestMessage {
    match msg.role.as_str() {
        "system" => ChatCompletionRequestSystemMessage::from(msg.content.as_str()).into(),

        "user" => ChatCompletionRequestUserMessage::from(msg.content.as_str()).into(),

        "tool" => ChatCompletionRequestToolMessage {
            content: msg.content.clone().into(),
            tool_call_id: msg.tool_call_id.clone().unwrap_or_default(),
        }.into(),

        "assistant" => {
            let tool_calls = if let (Some(name), Some(arguments)) = (&msg.tool_call_function, &msg.tool_call_arguments) {
                vec![ChatCompletionMessageToolCall {
                    id: msg.tool_call_id.clone().unwrap_or_default(),
                    function: FunctionCall {
                        name: name.clone(),
                        arguments: arguments.clone(),
                    },
                }]
            } else {
                Vec::new()
            };

            let tool_calls: Vec<ChatCompletionMessageToolCalls> = tool_calls
                .iter()
                .map(|tc| tc.clone().into())
                .collect();

            ChatCompletionRequestAssistantMessage {
                content: Some(ChatCompletionRequestAssistantMessageContent::from(msg.content.clone())),
                tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
                ..Default::default()
            }.into()
        },
        
        _ => panic!("Unknown message role: {}", msg.role),
    }
}

pub fn chat_message_to_sql_message(msg: &ChatCompletionRequestMessage, conversation_id: i64) -> aichats::SqlAiConversationMessage {
    match msg {
        ChatCompletionRequestMessage::System(system_msg) => aichats::SqlAiConversationMessage {
            id: 0,
            conversation_id,
            role: "system".to_owned(),
            content: match &system_msg.content {
                ChatCompletionRequestSystemMessageContent::Text(str) => str.clone(),
                _ => String::new(),
            },
            tool_call_id: None,
            tool_call_function: None,
            tool_call_arguments: None,
            timestamp: String::new(),
        },

        ChatCompletionRequestMessage::User(user_msg) => aichats::SqlAiConversationMessage {
            id: 0,
            conversation_id,
            role: "user".to_owned(),
            content: match &user_msg.content {
                ChatCompletionRequestUserMessageContent::Text(str) => str.clone(),
                _ => String::new(),
            },
            tool_call_id: None,
            tool_call_function: None,
            tool_call_arguments: None,
            timestamp: String::new(),
        },

        ChatCompletionRequestMessage::Tool(tool_msg) => aichats::SqlAiConversationMessage {
            id: 0,
            conversation_id,
            role: "tool".to_owned(),
            content: match &tool_msg.content {
                ChatCompletionRequestToolMessageContent::Text(str) => str.clone(),
                _ => String::new(),
            },
            tool_call_id: Some(tool_msg.tool_call_id.clone()),
            tool_call_function: None,
            tool_call_arguments: None,
            timestamp: String::new(),
        },

        ChatCompletionRequestMessage::Assistant(assistant_msg) => {
            let (tool_call_id, tool_call_function, tool_call_arguments) = assistant_msg.tool_calls.as_ref()
                .map_or((None, None, None), |tool_calls| if !tool_calls.is_empty() {
                    let tc = &tool_calls[0];
                    match tc {
                        ChatCompletionMessageToolCalls::Function(tool) => {
                            (
                                Some(tool.id.clone()),
                                Some(tool.function.name.clone()),
                                Some(tool.function.arguments.clone()),
                            )
                        },
                        _ => (None, None, None),
                    }
                } else {
                    (None, None, None)
                });

            aichats::SqlAiConversationMessage {
                id: 0,
                conversation_id,
                role: "assistant".to_owned(),
                content: match &assistant_msg.content {
                    Some(ChatCompletionRequestAssistantMessageContent::Text(str)) => str.clone(),
                    _ => String::new(),
                },
                tool_call_id,
                tool_call_function,
                tool_call_arguments,
                timestamp: String::new(),
            }
        },

        _ => panic!("Unknown ChatCompletionRequestMessage variant"),
    }
}