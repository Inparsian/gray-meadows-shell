use std::collections::HashMap;
use std::error::Error;
use async_openai::types::chat::ChatCompletionRequestMessage;

use crate::sql::wrappers::aichats;
use super::{sql, CHANNEL, SESSION, AIChannelMessage};

fn read_conversation(id: i64) -> Result<HashMap<i64, ChatCompletionRequestMessage>, Box<dyn Error>> {
    let sql_messages = aichats::get_messages(id)?;

    let mut chat_messages = HashMap::new();
    for msg in &sql_messages {
        let chat_msg = sql::sql_message_to_chat_message(msg);
        chat_messages.insert(msg.id, chat_msg);
    }

    Ok(chat_messages)
}

pub fn load_conversation(id: i64) {
    if let Some(session) = SESSION.get() {
        match aichats::get_conversation(id) {
            Ok(conv) => {
                let mut conversation = session.conversation.write().unwrap();
                match read_conversation(id) {
                    Ok(messages) => {
                        let mut msgs = session.messages.write().unwrap();
                        msgs.clear();
                        for (id, msg) in messages {
                            msgs.insert(id, msg);
                        }

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

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn rename_conversation(conversation_id: i64, new_title: &str) {
    if let Some(session) = SESSION.get() {
        if let Err(err) = aichats::rename_conversation(conversation_id, new_title) {
            eprintln!("Failed to rename AI chat conversation in database: {}", err);
            return;
        }

        {
            let mut conversation = session.conversation.write().unwrap();
            if let Some(current) = &mut *conversation && current.id == conversation_id {
                current.title = new_title.to_owned();
            }
        }

        if let Some(channel) = CHANNEL.get() {
            channel.spawn_send(AIChannelMessage::ConversationRenamed(conversation_id, new_title.to_owned()));
        }
    }
}

#[allow(dead_code)]
pub fn delete_conversation(conversation_id: i64) {
    if let Some(session) = SESSION.get() {
        if let Err(err) = aichats::delete_conversation(conversation_id) {
            eprintln!("Failed to delete AI chat conversation from database: {}", err);
            return;
        }

        {
            let mut conversation = session.conversation.write().unwrap();
            if let Some(current) = &*conversation && current.id == conversation_id {
                // Select a neighboring conversation if the deleted one is the current one
                match aichats::get_all_conversations() {
                    Ok(conversations) => {
                        if let Some(pos) = conversations.iter().position(|c| c.id == conversation_id) {
                            let new_conversation = if pos > 0 {
                                conversations.get(pos - 1)
                            } else {
                                conversations.get(pos + 1)
                            };
                            if let Some(new_conv) = new_conversation {
                                *conversation = Some(new_conv.clone());
                            } else {
                                *conversation = None;
                            }
                        }
                    },
                    Err(err) => {
                        eprintln!("Failed to get AI chat conversations from database: {}", err);
                        *conversation = None;
                    }
                }
            }
        }

        session.messages.write().unwrap().clear();

        if let Some(channel) = CHANNEL.get() {
            channel.spawn_send(AIChannelMessage::ConversationDeleted(conversation_id));
        }
    }
}