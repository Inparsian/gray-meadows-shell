// Non-OpenAI APIs that comply with the OpenAPI spec are not guaranteed to implement some
// of async-openai's default types correctly. Here the bare minimum is defined to ensure
// compatibility.

use std::pin::Pin;
use async_openai::error::OpenAIError;
use async_openai::types::chat::{ChatChoiceLogprobs, CompletionUsage, FinishReason, FunctionCallStream, FunctionType, Role};
use futures_lite::Stream;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ChatCompletionMessageToolCallChunk {
    pub id: Option<String>,
    pub r#type: Option<FunctionType>,
    pub function: Option<FunctionCallStream>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ChatCompletionStreamResponseDelta {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ChatCompletionMessageToolCallChunk>>,
    pub role: Option<Role>,
    pub refusal: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ChatChoiceStream {
    pub index: u32,
    pub delta: ChatCompletionStreamResponseDelta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<ChatChoiceLogprobs>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct CreateChatCompletionStreamResponse {
    pub choices: Vec<ChatChoiceStream>,
    pub created: u32,
    pub model: String,
    pub object: String,
    pub usage: Option<CompletionUsage>,
}

pub type ChatCompletionResponseStream = Pin<Box<dyn Stream<Item = Result<CreateChatCompletionStreamResponse, OpenAIError>> + Send>>;