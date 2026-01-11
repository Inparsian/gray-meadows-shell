pub mod openai;
pub mod gemini;

use std::pin::Pin;
use std::sync::{Arc, RwLock};

use crate::utils::broadcast::BroadcastChannel;
use super::{AiChannelMessage, AiConversationItem, AiConversationItemPayload};

pub struct AiServiceResult {
    pub items: Vec<AiConversationItemPayload>,
    pub should_request_more: bool,
}

pub trait AiService: Send + Sync {
    fn service_name(&self) -> String;

    fn make_stream_request(
        &self,
        items: Vec<AiConversationItem>,
        channel: &BroadcastChannel<AiChannelMessage>,
        stop_cycle_flag: Arc<RwLock<bool>>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<AiServiceResult>> + 'static + Send>>;
}