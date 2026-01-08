use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use futures::StreamExt as _;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::error::OpenAIError;
use async_openai::types::responses::{
    AssistantRole,
    CreateResponseArgs, 
    FunctionCallOutput, FunctionCallOutputItemParam, FunctionTool, FunctionToolCall,
    InputContent, InputMessage, InputRole, InputTextContent,
    Item, MessageItem,
    OutputContent, OutputItem, OutputMessage, OutputMessageContent, OutputStatus, OutputTextContent,
    Reasoning, ReasoningEffort, ReasoningItem, ReasoningSummary,
    ResponseStream, ResponseStreamEvent,
    ServiceTier,
    Summary, SummaryPart,
    Tool
};

use crate::broadcast::BroadcastChannel;
use crate::config::read_config;
use super::super::variables::transform_variables;
use super::super::{AiChannelMessage, AiConversationItem, AiConversationItemPayload, AiConversationDelta};
use super::super::types::AiFunction;
use super::super::tools;

#[derive(Debug, Default, Clone)]
pub struct OpenAiService {
    pub client: Arc<RwLock<Option<Client<OpenAIConfig>>>>,
}

impl OpenAiService {
    fn make_client(&self) {
        let app_config = read_config();
        let config = OpenAIConfig::new()
            .with_api_key(app_config.ai.openai.api_key.as_str());

        self.client.write().unwrap().replace(Client::with_config(config));
    }

    async fn create_stream(&self, items: Vec<AiConversationItem>) -> Result<ResponseStream, OpenAIError> {
        if self.client.read().unwrap().is_none() {
            self.make_client();
        }

        let client = {
            let client_guard = self.client.read().unwrap();
            client_guard.as_ref().unwrap().clone()
        };

        let app_config = read_config().clone();
        let mut native_items = items
            .clone()
            .iter_mut()
            .map(|item| {
                if app_config.ai.user_message_timestamps {
                    item.inject_timestamp_into_content();
                }
                
                Self::transform_item_into_native(item.clone()).unwrap()
            })
            .collect::<Vec<Item>>();

        native_items.insert(0, Item::Message(MessageItem::Input(InputMessage {
            role: InputRole::Developer,
            content: vec![InputContent::InputText(InputTextContent {
                text: transform_variables(&app_config.ai.prompt),
            })],
            status: None,
        })));

        let tools = tools::get_tools()?
            .into_iter()
            .map(Self::transform_function_into_tool)
            .collect::<Vec<Tool>>();

        let request = if matches!(app_config.ai.openai.reasoning_effort.as_str(), "minimal" | "low" | "medium" | "high" | "xhigh") {
            CreateResponseArgs::default()
                .max_output_tokens(2048_u32)
                .stream(true)
                .model(app_config.ai.openai.model.as_str())
                .service_tier(match app_config.ai.openai.service_tier.as_str() {
                    "flex" => ServiceTier::Flex,
                    "priority" => ServiceTier::Priority,
                    _ => ServiceTier::Default,
                })
                .reasoning(Reasoning {
                    effort: Some(match app_config.ai.openai.reasoning_effort.as_str() {
                        "minimal" => ReasoningEffort::Minimal,
                        "low" => ReasoningEffort::Low,
                        "medium" => ReasoningEffort::Medium,
                        "high" => ReasoningEffort::High,
                        "xhigh" => ReasoningEffort::Xhigh,
                        _ => ReasoningEffort::None,
                    }),
                    summary: Some(ReasoningSummary::Auto),
                })
                .tools(tools)
                .input(native_items)
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
                .tools(tools)
                .input(native_items)
                .build()?
        };

        client.responses().create_stream(request).await
    }

    fn transform_item_into_native(item: AiConversationItem) -> Option<Item> {
        match item.payload {
            AiConversationItemPayload::Message { id, role, content, .. } => {
                if role == "assistant" {
                    Some(Item::Message(MessageItem::Output(OutputMessage {
                        content: vec![OutputMessageContent::OutputText(OutputTextContent {
                            text: content,
                            annotations: vec![],
                            logprobs: None,
                        })],
                        role: AssistantRole::Assistant,
                        id,
                        status: OutputStatus::Completed,
                    })))
                } else {
                    Some(Item::Message(MessageItem::Input(InputMessage {
                        content: vec![InputContent::InputText(InputTextContent {
                            text: content,
                        })],
                        role: if role == "user" { InputRole::User } else { InputRole::Developer },
                        status: None,
                    })))
                }
            },

            AiConversationItemPayload::Reasoning { id, summary, encrypted_content } => {
                Some(Item::Reasoning(ReasoningItem {
                    id,
                    summary: vec![SummaryPart::SummaryText(Summary {
                        text: summary,
                    })],
                    content: None,
                    encrypted_content: Some(encrypted_content),
                    status: None,
                }))
            },

            AiConversationItemPayload::FunctionCall { id, name, arguments, call_id, .. } => {
                Some(Item::FunctionCall(FunctionToolCall {
                    id: Some(id),
                    name,
                    arguments,
                    call_id,
                    status: None,
                }))
            },

            AiConversationItemPayload::FunctionCallOutput { call_id, output, .. } => {
                Some(Item::FunctionCallOutput(FunctionCallOutputItemParam {
                    call_id,
                    output: FunctionCallOutput::Text(output),
                    id: None,
                    status: None,
                }))
            },
        }
    }

    fn transform_native_into_item(native_item: Item) -> Option<AiConversationItem> {
        let now = chrono::Utc::now().format(super::super::TIMESTAMP_FORMAT).to_string();

        let payload = match native_item {
            Item::Message(msg) => {
                match msg {
                    MessageItem::Input(input_msg) => Some(AiConversationItemPayload::Message {
                        id: String::new(),
                        role: match input_msg.role {
                            InputRole::User => "user".to_owned(),
                            _ => "developer".to_owned(),
                        },
                        content: match &input_msg.content.first() {
                            Some(InputContent::InputText(text_content)) => text_content.text.clone(),
                            _ => String::new(),
                        },
                        thought_signature: None,
                    }),

                    MessageItem::Output(output_msg) => Some(AiConversationItemPayload::Message {
                        id: output_msg.id.clone(),
                        role: "assistant".to_owned(),
                        content: match &output_msg.content.first() {
                            Some(OutputMessageContent::OutputText(text_content)) => text_content.text.clone(),
                            _ => String::new(),
                        },
                        thought_signature: None,
                    }),
                }
            },

            Item::Reasoning(reasoning) => Some(AiConversationItemPayload::Reasoning {
                id: reasoning.id.clone(),
                summary: reasoning.summary.iter().map(|part| {
                    let SummaryPart::SummaryText(summary) = part;
                    summary.text.clone()
                }).collect::<Vec<String>>().join("\n\n"),
                encrypted_content: reasoning.encrypted_content.clone().unwrap_or_default(),
            }),

            Item::FunctionCall(func_call) => Some(AiConversationItemPayload::FunctionCall {
                id: func_call.id.clone().unwrap_or_default(),
                name: func_call.name.clone(),
                arguments: func_call.arguments.clone(),
                call_id: func_call.call_id,
                thought_signature: None,
            }),

            Item::FunctionCallOutput(func_output) => Some(AiConversationItemPayload::FunctionCallOutput {
                call_id: func_output.call_id.clone(),
                output: match &func_output.output {
                    FunctionCallOutput::Text(text) => text.clone(),
                    _ => String::new(),
                },
                name: None,
            }),

            _ => None,
        };

        payload.map(|p| AiConversationItem {
            id: 0,
            conversation_id: 0,
            payload: p,
            timestamp: Some(now),
        })
    }

    fn transform_function_into_tool(func: AiFunction) -> Tool {
        Tool::Function(FunctionTool {
            name: func.name,
            description: Some(func.description),
            strict: Some(func.strict),
            parameters: Some(func.schema),
        })
    }
}

impl super::AiService for OpenAiService {
    fn service_name(&self) -> String {
        "openai".to_owned()
    }

    fn make_stream_request(
        &self,
        items: Vec<AiConversationItem>,
        channel: &BroadcastChannel<AiChannelMessage>,
        stop_cycle_flag: Arc<RwLock<bool>>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<super::AiServiceResult>> + 'static + Send>> {
        let channel = channel.clone();
        let client = self.client.clone();

        Box::pin(async move {
            let service = OpenAiService {
                client
            };

            let mut should_request_more = true;
            let mut new_items: HashMap<String, Item> = HashMap::new();
            let mut stream = service.create_stream(items).await?;

            channel.send(AiChannelMessage::StreamStart).await;

            while let Some(event) = stream.next().await {
                if *stop_cycle_flag.read().unwrap() {
                    break;
                }

                match event? {
                    ResponseStreamEvent::ResponseOutputItemAdded(event) => {
                        match event.item {
                            OutputItem::Message(output_message) => {
                                let id = output_message.id.clone();
                                new_items.insert(id.clone(), Item::Message(MessageItem::Output(OutputMessage {
                                    id,
                                    role: AssistantRole::Assistant,
                                    content: vec![],
                                    status: OutputStatus::InProgress,
                                })));
                            },

                            OutputItem::Reasoning(output_reasoning) => {
                                let id = output_reasoning.id.clone();
                                new_items.insert(id.clone(), Item::Reasoning(ReasoningItem {
                                    id,
                                    summary: vec![],
                                    content: None,
                                    encrypted_content: None,
                                    status: Some(OutputStatus::InProgress),
                                }));
                            },

                            OutputItem::FunctionCall(output_function_call) => {
                                let id = output_function_call.id.clone().unwrap_or_default();
                                new_items.insert(id.clone(), Item::FunctionCall(FunctionToolCall {
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
                        let id = event.item_id.clone();
                        match event.part {
                            OutputContent::OutputText(content) => {
                                if let Some(Item::Message(MessageItem::Output(output_message))) = new_items.get_mut(&id) {
                                    output_message.content.push(OutputMessageContent::OutputText(OutputTextContent {
                                        text: content.text.clone(),
                                        annotations: vec![],
                                        logprobs: None,
                                    }));
                                }
                            },

                            OutputContent::ReasoningText(content) => {
                                if let Some(Item::Reasoning(reasoning_item)) = new_items.get_mut(&id) {
                                    reasoning_item.summary.push(SummaryPart::SummaryText(Summary {
                                        text: content.text.clone(),
                                    }));
                                }
                            },

                            _ => {},
                        }
                    },

                    ResponseStreamEvent::ResponseOutputTextDelta(event) => {
                        let id = event.item_id.clone();
                        match new_items.get_mut(&id) {
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

                                channel.send(AiChannelMessage::StreamChunk(AiConversationDelta::Message(event.delta.clone()))).await;
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

                    ResponseStreamEvent::ResponseReasoningSummaryPartAdded(event) => {
                        let id = event.item_id.clone();
                        if let Some(Item::Reasoning(reasoning_item)) = new_items.get_mut(&id) {
                            let SummaryPart::SummaryText(summary) = &event.part;

                            reasoning_item.summary.push(SummaryPart::SummaryText(summary.clone()));
                        }

                        channel.send(AiChannelMessage::StreamReasoningSummaryPartAdded).await;
                    }

                    ResponseStreamEvent::ResponseReasoningSummaryTextDelta(event) => {
                        let id = event.item_id.clone();
                        if let Some(Item::Reasoning(reasoning_item)) = new_items.get_mut(&id) {
                            if let Some(SummaryPart::SummaryText(last_summary)) = reasoning_item.summary.last_mut() {
                                last_summary.text.push_str(&event.delta);
                            } else {
                                reasoning_item.summary.push(SummaryPart::SummaryText(Summary {
                                    text: event.delta.clone(),
                                }));
                            }

                            channel.send(AiChannelMessage::StreamChunk(AiConversationDelta::Reasoning(event.delta.clone()))).await;
                        }
                    },

                    ResponseStreamEvent::ResponseOutputItemDone(event) => {
                        let id = match &event.item {
                            OutputItem::Message(msg) => msg.id.clone(),
                            OutputItem::Reasoning(reasoning) => reasoning.id.clone(),
                            OutputItem::FunctionCall(func_call) => func_call.id.clone().unwrap_or_default(),
                            _ => String::new(),
                        };

                        if let Some(item) = new_items.get_mut(&id) {
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

                                    // Do not request again if a power action is being performed
                                    // Because users quite literally can not see AI responses if their
                                    // system is powered off
                                    if func_call.name == "perform_power_action" {
                                        should_request_more = false;
                                    }

                                    channel.send(AiChannelMessage::ToolCall(
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

            if let Ok(mut stop_flag) = stop_cycle_flag.write() {
                *stop_flag = false;
            }

            let mut transformed_items = new_items.values()
                .cloned()
                .filter_map(Self::transform_native_into_item)
                .collect::<Vec<AiConversationItem>>();

            // Ensure that this is properly sorted so the API does not yell at us later
            transformed_items.sort_by_key(|item| {
                match &item.payload {
                    AiConversationItemPayload::Reasoning { .. } => 0,
                    AiConversationItemPayload::Message { role, .. } if role == "assistant" => 1,
                    AiConversationItemPayload::FunctionCall { .. } => 2,
                    _ => 3,
                }
            });

            let has_tool_calls = transformed_items.iter().any(|item| {
                matches!(item.payload, AiConversationItemPayload::FunctionCall { .. })
            });

            // Go for another request after tool execution, in case the AI wants to say
            // something after tool execution or perform more tool calls
            Ok(super::AiServiceResult {
                items: transformed_items,
                should_request_more: has_tool_calls && should_request_more,
            })
        })
    }
}