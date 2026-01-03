use async_openai::types::responses::{
    AssistantRole,
    FunctionCallOutput, FunctionCallOutputItemParam, FunctionToolCall,
    InputContent, InputMessage, InputRole, InputTextContent,
    Item, MessageItem,
    OutputMessage, OutputMessageContent, OutputStatus, OutputTextContent,
    ReasoningItem,
    Summary, SummaryPart
};

use crate::sql::wrappers::aichats::{SqlAiConversationItem, SqlAiConversationItemPayload};

pub fn sql_item_to_item(item: &SqlAiConversationItemPayload) -> Option<Item> {
    match item {
        SqlAiConversationItemPayload::Message { id, role, content, .. } => {
            if role == "assistant" {
                Some(Item::Message(MessageItem::Output(OutputMessage {
                    content: vec![OutputMessageContent::OutputText(OutputTextContent {
                        text: content.clone(),
                        annotations: vec![],
                        logprobs: None,
                    })],
                    role: AssistantRole::Assistant,
                    id: id.clone(),
                    status: OutputStatus::Completed,
                })))
            } else {
                Some(Item::Message(MessageItem::Input(InputMessage {
                    content: vec![InputContent::InputText(InputTextContent {
                        text: content.clone(),
                    })],
                    role: if role == "user" { InputRole::User } else { InputRole::Developer },
                    status: None,
                })))
            }
        },

        SqlAiConversationItemPayload::Reasoning { id, summary, encrypted_content } => {
            Some(Item::Reasoning(ReasoningItem {
                id: id.clone(),
                summary: vec![SummaryPart::SummaryText(Summary {
                    text: summary.clone(),
                })],
                content: None,
                encrypted_content: Some(encrypted_content.clone()),
                status: None,
            }))
        },

        SqlAiConversationItemPayload::FunctionCall { id, name, arguments, call_id } => {
            Some(Item::FunctionCall(FunctionToolCall {
                id: Some(id.clone()),
                name: name.clone(),
                arguments: arguments.clone(),
                call_id: call_id.clone(),
                status: None,
            }))
        },

        SqlAiConversationItemPayload::FunctionCallOutput { call_id, output } => {
            Some(Item::FunctionCallOutput(FunctionCallOutputItemParam {
                call_id: call_id.clone(),
                output: FunctionCallOutput::Text(output.clone()),
                id: None,
                status: None,
            }))
        },
    }
}

pub fn item_to_sql_item(item: &Item, conversation_id: i64) -> Option<SqlAiConversationItem> {
    let now = chrono::Utc::now().format(super::TIMESTAMP_FORMAT).to_string();

    let payload = match item {
        Item::Message(msg) => {
            match msg {
                MessageItem::Input(input_msg) => Some(SqlAiConversationItemPayload::Message {
                    id: String::new(),
                    role: match input_msg.role {
                        InputRole::User => "user".to_owned(),
                        _ => "developer".to_owned(),
                    },
                    content: match &input_msg.content.first() {
                        Some(InputContent::InputText(text_content)) => text_content.text.clone(),
                        _ => String::new(),
                    },
                }),

                MessageItem::Output(output_msg) => Some(SqlAiConversationItemPayload::Message {
                    id: output_msg.id.clone(),
                    role: "assistant".to_owned(),
                    content: match &output_msg.content.first() {
                        Some(OutputMessageContent::OutputText(text_content)) => text_content.text.clone(),
                        _ => String::new(),
                    },
                }),
            }
        },

        Item::Reasoning(reasoning) => Some(SqlAiConversationItemPayload::Reasoning {
            id: reasoning.id.clone(),
            summary: reasoning.summary.first().map(|part| {
                let SummaryPart::SummaryText(summary) = part;
                summary.text.clone()
            }).unwrap_or_default(),
            encrypted_content: reasoning.encrypted_content.clone().unwrap_or_default(),
        }),

        Item::FunctionCall(func_call) => Some(SqlAiConversationItemPayload::FunctionCall {
            id: func_call.id.clone().unwrap_or_default(),
            name: func_call.name.clone(),
            arguments: func_call.arguments.clone(),
            call_id: func_call.call_id.clone(),
        }),

        Item::FunctionCallOutput(func_output) => Some(SqlAiConversationItemPayload::FunctionCallOutput {
            call_id: func_output.call_id.clone(),
            output: match &func_output.output {
                FunctionCallOutput::Text(text) => text.clone(),
                _ => String::new(),
            },
        }),

        _ => None,
    };

    payload.map(|p| SqlAiConversationItem {
        id: 0,
        conversation_id,
        payload: p,
        timestamp: Some(now),
    })
}