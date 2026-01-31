use std::error::Error;

use crate::sql::wrappers::aichats;
use super::{CHANNEL, SESSION, AiChannelMessage};
use super::types::AiConversationItem;

async fn read_conversation(id: i64) -> Result<Vec<AiConversationItem>, Box<dyn Error>> {
    let mut sql_items = aichats::get_items(id).await?;
    sql_items.sort_by_key(|item| item.id);
    Ok(sql_items)
}

pub async fn load_conversation(id: i64) {
    if let Some(session) = SESSION.get() {
        match aichats::get_conversation(id).await {
            Ok(conv) => {
                match read_conversation(id).await {
                    Ok(items) => {
                        {
                            let mut conversation = session.conversation.write().unwrap();
                            *session.items.write().unwrap() = items;
                            *conversation = Some(conv);
                            if let Some(channel) = CHANNEL.get() {
                                channel.spawn_send(AiChannelMessage::ConversationLoaded(conversation.as_ref().unwrap().clone()));
                            }
                        }

                        if let Err(err) = aichats::set_state_conversation_id(Some(id)).await {
                            error!(%err, "Failed to update current AI chat conversation in database");
                        }
                    },
                
                    Err(err) => {
                        error!(%err, "Failed to load AI chat conversation from database");
                    }
                }
            },

            Err(err) => {
                error!(%err, "Failed to load AI chat conversation info from database");
            }
        }
    }
}

pub async fn load_first_conversation() {
    if let Some(first_conversation) = aichats::get_all_conversations()
        .await
        .unwrap_or_default()
        .first()
    {
        load_conversation(first_conversation.id).await;
    }
}

pub async fn add_conversation(title: &str) {
    match aichats::add_conversation(title).await {
        Ok(conversation_id) => {
            match aichats::get_conversation(conversation_id).await {
                Ok(conversation) => if let Some(channel) = CHANNEL.get() {
                    channel.spawn_send(AiChannelMessage::ConversationAdded(conversation));
                },

                Err(err) => {
                    error!(%err, "Failed to load newly added AI chat conversation from database");
                }
            }
        },

        Err(err) => {
            error!(%err, "Failed to add AI chat conversation to database");
        }
    }
}

pub async fn rename_conversation(conversation_id: i64, new_title: &str) {
    if let Some(session) = SESSION.get() {
        if let Err(err) = aichats::rename_conversation(conversation_id, new_title).await {
            error!(%err, "Failed to rename AI chat conversation in database");
            return;
        }

        let mut conversation = session.conversation.write().unwrap();
        if let Some(current) = &mut *conversation && current.id == conversation_id {
            current.title = new_title.to_owned();
        }

        if let Some(channel) = CHANNEL.get() {
            channel.spawn_send(AiChannelMessage::ConversationRenamed(conversation_id, new_title.to_owned()));
        }
    }
}

pub async fn delete_conversation(conversation_id: i64) {
    if let Some(session) = SESSION.get() {
        if let Err(err) = aichats::delete_conversation(conversation_id).await {
            error!(%err, "Failed to delete AI chat conversation from database");
            return;
        }

        let current_conversation_id = session.conversation.read().unwrap().as_ref().map(|c| c.id);
        if current_conversation_id == Some(conversation_id) {
            load_first_conversation().await;
        }

        if let Some(channel) = CHANNEL.get() {
            channel.spawn_send(AiChannelMessage::ConversationDeleted(conversation_id));
        }
    }
}

pub async fn clear_conversation(conversation_id: i64) {
    // trim to 0
    if let Err(err) = aichats::trim_items(conversation_id, 0).await {
        error!(%err, "Failed to clear AI chat conversation items from database");
        return;
    }

    load_conversation(conversation_id).await;
}