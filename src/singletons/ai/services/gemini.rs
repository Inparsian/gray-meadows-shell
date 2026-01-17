use std::pin::Pin;
use std::sync::{Arc, RwLock};
use futures::StreamExt as _;
use gemini_rust::{
    Blob,
    Content, ContentBuilder,
    FunctionCall,
    Gemini, Part, Role,
    ThinkingConfig, ThinkingLevel,
};

use crate::config::{AiService as AiConfigService, GeminiThinkingLevel, read_config};
use crate::utils::broadcast::BroadcastChannel;
use crate::singletons::ai::images::load_image_data;
use crate::singletons::ai::tools::gemini::add_gemini_tools;
use super::super::variables::transform_variables;
use super::super::{AiChannelMessage, AiConversationItem, AiConversationItemPayload, AiConversationDelta};

#[derive(Default, Debug, Clone)]
pub struct GeminiContext {
    pub reasoning: Option<AiConversationItemPayload>,
    pub response: Option<AiConversationItemPayload>,
    pub tool_calls: Vec<AiConversationItemPayload>,
}

#[derive(Clone)]
pub struct GeminiService;

impl GeminiService {
    fn transform_items_into_builder(items: Vec<AiConversationItem>, client: &Gemini) -> ContentBuilder {
        let mut assistant_parts: Vec<Part> = vec![];
        let mut user_parts: Vec<Part> = vec![];
        let mut builder = client.generate_content();

        let flush_assistant_parts = |assistant_parts: &mut Vec<Part>, builder: &mut ContentBuilder| {
            if !assistant_parts.is_empty() {
                builder.contents.push(Content {
                    parts: Some(std::mem::take(assistant_parts)),
                    role: Some(Role::Model)
                });
            }
        };

        let flush_user_parts = |user_parts: &mut Vec<Part>, builder: &mut ContentBuilder| {
            if !user_parts.is_empty() {
                builder.contents.push(Content {
                    parts: Some(std::mem::take(user_parts)),
                    role: Some(Role::User)
                });
            }
        };

        for item in items {
            match &item.payload {
                AiConversationItemPayload::Message { role, content, thought_signature, .. } => {
                    let role = match role.as_str() {
                        "system" | "developer" => "system",
                        "assistant" => "assistant",
                        _ => "user",
                    };

                    match role {
                        "assistant" => {
                            flush_user_parts(&mut user_parts, &mut builder);
                            assistant_parts.push(Part::Text {
                                text: content.clone(),
                                thought: Some(false),
                                thought_signature: thought_signature.clone(),
                            });
                        },

                        "user" => {
                            flush_assistant_parts(&mut assistant_parts, &mut builder);
                            user_parts.push(Part::Text {
                                text: content.clone(),
                                thought: Some(false),
                                thought_signature: None,
                            });
                        },

                        "system" => {
                            flush_assistant_parts(&mut assistant_parts, &mut builder);
                            flush_user_parts(&mut user_parts, &mut builder);
                            builder = builder.with_system_prompt(content);
                        },

                        _ => unreachable!(),
                    }
                },

                AiConversationItemPayload::Image { uuid, .. } => if let Ok(data) = load_image_data(uuid) {
                    flush_assistant_parts(&mut assistant_parts, &mut builder);
                    
                    user_parts.push(Part::InlineData {
                        inline_data: Blob {
                            mime_type: "image/png".to_owned(),
                            data,
                        },
                        media_resolution: None,
                    });
                },

                AiConversationItemPayload::Reasoning { summary, encrypted_content, .. } => {
                    flush_user_parts(&mut user_parts, &mut builder);
                    let thought_signature = if encrypted_content.is_empty() {
                        None
                    } else {
                        Some(encrypted_content.clone())
                    };

                    assistant_parts.push(Part::Text {
                        text: summary.clone(),
                        thought: Some(true),
                        thought_signature,
                    });
                },

                AiConversationItemPayload::FunctionCall { name, arguments, thought_signature, .. } => {
                    flush_user_parts(&mut user_parts, &mut builder);
                    let json_arguments = serde_json::from_str::<serde_json::Value>(arguments)
                        .unwrap_or(serde_json::Value::Null);

                    let function_call = Part::FunctionCall {
                        function_call: FunctionCall {
                            name: name.clone(),
                            args: json_arguments,
                            thought_signature: None,
                        },
                        // This is required for context.
                        thought_signature: thought_signature.clone(),
                    };

                    assistant_parts.push(function_call);
                },

                AiConversationItemPayload::FunctionCallOutput { name, output, .. } => {
                    flush_user_parts(&mut user_parts, &mut builder);
                    flush_assistant_parts(&mut assistant_parts, &mut builder);

                    let json_output = serde_json::from_str::<serde_json::Value>(output)
                        .unwrap_or(serde_json::Value::Null);

                    if let Ok(new_builder) = builder.clone().with_function_response(
                        name.clone().unwrap_or_default(), 
                        json_output
                    ) {
                        builder = new_builder;
                    }
                },
            }
        }

        flush_user_parts(&mut user_parts, &mut builder);

        builder
    }
}

impl super::AiService for GeminiService {
    fn service(&self) -> AiConfigService {
        AiConfigService::Gemini
    }

    fn make_stream_request(
        &self,
        items: Vec<AiConversationItem>,
        channel: &BroadcastChannel<AiChannelMessage>,
        stop_cycle_flag: Arc<RwLock<bool>>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<super::AiServiceResult>> + 'static + Send>> {
        let channel = channel.clone();
        let config = read_config();

        let api_key = config.ai.gemini.api_key.clone();
        let model = config.ai.gemini.model.clone();
        let system_prompt = config.ai.prompt.clone();
        let thinking_budget = config.ai.gemini.thinking_budget as i32;
        let thinking_level = config.ai.gemini.thinking_level.clone();

        Box::pin(async move {
            let client = Gemini::with_model(api_key, format!("models/{}", model))
                .expect("Failed to create Gemini client");

            let mut builder = Self::transform_items_into_builder(items, &client)
                .with_system_prompt(transform_variables(system_prompt.as_str()))
                .with_thinking_config(ThinkingConfig {
                    thinking_budget: (!matches!(thinking_level, GeminiThinkingLevel::Low | GeminiThinkingLevel::High))
                        .then_some(thinking_budget),
                    include_thoughts: Some(true),
                    thinking_level: match thinking_level {
                        GeminiThinkingLevel::Low => Some(ThinkingLevel::Low),
                        GeminiThinkingLevel::High => Some(ThinkingLevel::High),
                        _ => None,
                    },
                });
            
            builder = add_gemini_tools(builder);

            let mut should_request_more = true;
            let mut context = GeminiContext::default();

            channel.send(AiChannelMessage::StreamStart).await;
            let mut stream = builder.execute_stream().await?;
            while let Some(chunk) = stream.next().await {
                if *stop_cycle_flag.read().unwrap() {
                    break;
                }

                let result = chunk?;
                let candidate_parts = result.candidates.first()
                    .and_then(|c| c.content.parts.clone())
                    .unwrap_or_default();

                for part in candidate_parts {
                    match part {
                        Part::Text { text, thought, thought_signature } => if thought == Some(true) {
                            let reasoning_payload = AiConversationItemPayload::Reasoning {
                                id: String::new(),
                                summary: text.clone(),
                                encrypted_content: thought_signature.clone().unwrap_or_default(),
                            };

                            if let Some(reasoning) = &mut context.reasoning {
                                if let AiConversationItemPayload::Reasoning { summary, .. } = reasoning {
                                    summary.push_str(&text);
                                }
                            } else {
                                context.reasoning = Some(reasoning_payload.clone());
                            }

                            channel.send(AiChannelMessage::StreamChunk(AiConversationDelta::Reasoning(text.clone()))).await;
                        } else {
                            let message_payload = AiConversationItemPayload::Message {
                                id: String::new(),
                                role: "assistant".to_owned(),
                                content: text.clone(),
                                thought_signature: thought_signature.clone(),
                            };

                            if let Some(response) = &mut context.response {
                                if let AiConversationItemPayload::Message { content, .. } = response {
                                    content.push_str(&text);
                                }

                                if let Some(thought_signature) = thought_signature.clone()
                                    && let AiConversationItemPayload::Message { thought_signature: resp_thought_sig, .. } = response
                                {
                                    *resp_thought_sig = Some(thought_signature);
                                }
                            } else {
                                context.response = Some(message_payload.clone());
                            }

                            channel.send(AiChannelMessage::StreamChunk(AiConversationDelta::Message(text.clone()))).await;
                        },

                        Part::FunctionCall { function_call, thought_signature } => {
                            let function_call_payload = AiConversationItemPayload::FunctionCall {
                                id: String::new(),
                                name: function_call.name.clone(),
                                arguments: function_call.args.to_string(),
                                call_id: String::new(),
                                thought_signature: thought_signature.clone(),
                            };

                            // Do not request again if a power action is being performed
                            // Because users quite literally can not see AI responses if their
                            // system is powered off
                            if function_call.name == "perform_power_action" {
                                should_request_more = false;
                            }

                            context.tool_calls.push(function_call_payload.clone());

                            channel.send(AiChannelMessage::ToolCall(
                                function_call.name.clone(),
                                function_call.args.to_string(),
                            )).await;
                        }

                        _ => {}
                    }
                }
            }

            if let Ok(mut stop_flag) = stop_cycle_flag.write() {
                *stop_flag = false;
            }

            // Flatten context into result items
            let mut items: Vec<AiConversationItemPayload> = vec![];
            if let Some(reasoning) = context.reasoning {
                items.push(reasoning);
            }

            if let Some(response) = context.response {
                items.push(response);
            }

            for tool_call in &context.tool_calls {
                items.push(tool_call.clone());
            }

            // Go for another request after tool execution, in case the AI wants to say
            // something after tool execution or perform more tool calls
            Ok(super::AiServiceResult {
                items,
                should_request_more: !context.tool_calls.is_empty() && should_request_more,
            })
        })
    }
}