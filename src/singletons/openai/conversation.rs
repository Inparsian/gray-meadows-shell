use std::error::Error;

use crate::sql::wrappers::aichats;
use super::{sql, CHANNEL, SESSION, AIChannelMessage, AISessionMessage};

fn read_conversation(id: i64) -> Result<Vec<AISessionMessage>, Box<dyn Error>> {
    let sql_messages = aichats::get_messages(id)?;

    let mut chat_messages = Vec::new();
    for msg in &sql_messages {
        let Some(chat_msg) = sql::sql_message_to_chat_message(msg) else {
            continue;
        };

        chat_messages.push(AISessionMessage {
            id: msg.id,
            timestamp: msg.timestamp.clone(),
            message: chat_msg,
        });
    }

    chat_messages.sort_by_key(|msg| msg.id);
    Ok(chat_messages)
}

pub fn load_conversation(id: i64) {
    if let Some(session) = SESSION.get() {
        match aichats::get_conversation(id) {
            Ok(conv) => {
                let mut conversation = session.conversation.write().unwrap();
                match read_conversation(id) {
                    Ok(messages) => {
                        *session.messages.write().unwrap() = messages;
                        *conversation = Some(conv);
                        if let Some(channel) = CHANNEL.get() {
                            channel.spawn_send(AIChannelMessage::ConversationLoaded(conversation.as_ref().unwrap().clone()));
                        }
                    },
                
                    Err(err) => {
                        eprintln!("Failed to load AI chat conversation from database: {}", err);
                    }
                }
            },

            Err(err) => {
                eprintln!("Failed to load AI chat conversation info from database: {}", err);
            }
        }
    }
}

pub fn load_first_conversation() {
    if let Some(first_conversation) = aichats::get_all_conversations()
        .unwrap_or_default()
        .first()
    {
        load_conversation(first_conversation.id);
    }
}

pub fn add_conversation(title: &str) {
    match aichats::add_conversation(title) {
        Ok(conversation_id) => {
            match aichats::get_conversation(conversation_id) {
                Ok(conversation) => {
                    if let Some(channel) = CHANNEL.get() {
                        channel.spawn_send(AIChannelMessage::ConversationAdded(conversation));
                    }
                },

                Err(err) => {
                    eprintln!("Failed to load newly added AI chat conversation from database: {}", err);
                }
            }
        },

        Err(err) => {
            eprintln!("Failed to add AI chat conversation to database: {}", err);
        }
    }
}

pub fn rename_conversation(conversation_id: i64, new_title: &str) {
    if let Some(session) = SESSION.get() {
        if let Err(err) = aichats::rename_conversation(conversation_id, new_title) {
            eprintln!("Failed to rename AI chat conversation in database: {}", err);
            return;
        }

        let mut conversation = session.conversation.write().unwrap();
        if let Some(current) = &mut *conversation && current.id == conversation_id {
            current.title = new_title.to_owned();
        }

        if let Some(channel) = CHANNEL.get() {
            channel.spawn_send(AIChannelMessage::ConversationRenamed(conversation_id, new_title.to_owned()));
        }
    }
}

pub fn delete_conversation(conversation_id: i64) {
    if let Some(session) = SESSION.get() {
        if let Err(err) = aichats::delete_conversation(conversation_id) {
            eprintln!("Failed to delete AI chat conversation from database: {}", err);
            return;
        }

        let current_conversation_id = session.conversation.read().unwrap().as_ref().map(|c| c.id);
        if current_conversation_id == Some(conversation_id) {
            load_first_conversation();
        }

        if let Some(channel) = CHANNEL.get() {
            channel.spawn_send(AIChannelMessage::ConversationDeleted(conversation_id));
        }
    }
}

pub fn clear_conversation(conversation_id: i64) {
    // trim to 0
    if let Err(err) = aichats::trim_messages(conversation_id, 0) {
        eprintln!("Failed to clear AI chat conversation messages from database: {}", err);
        return;
    }

    load_conversation(conversation_id);
}