use serde::{Serialize, Deserialize};

use crate::SQL_CONNECTION;
use super::super::last_insert_rowid;

#[derive(Debug, Clone)]
pub struct SqlAiConversation {
    pub id: i64,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SqlAiConversationItemPayload {
    Message {
        id: String,
        role: String,
        content: String,
    },

    Reasoning {
        id: String,
        summary: String,
        encrypted_content: String,
    },

    FunctionCall {
        id: String,
        name: String,
        arguments: String,
        call_id: String,
    },

    FunctionCallOutput {
        call_id: String,
        output: String,
    },
}

pub struct SqlAiConversationItem {
    pub id: i64,
    pub conversation_id: i64,
    pub payload: SqlAiConversationItemPayload,
    pub timestamp: Option<String>,
}

/// Ensures there is at least one AI chat conversation in the database.
pub fn ensure_default_conversation() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let mut cursor = connection.prepare("SELECT COUNT(*) FROM aichat_conversations")?;
        if cursor.next()? == sqlite::State::Row {
            let count = cursor.read::<i64, _>(0)?;
            if count == 0 {
                connection.execute("INSERT INTO aichat_conversations (title) VALUES ('Default Conversation')")?;
            }
        }
        Ok(())
    } else {
        Err("No database connection available".into())
    }
}

/// Adds an item to the specified AI chat conversation and returns its new ID.
pub fn add_item(item: &SqlAiConversationItem) -> Result<i64, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let statement = format!(
            "INSERT INTO aichat_items (conversation_id, payload) \
             VALUES ({}, '{}')",
            item.conversation_id,
            serde_json::to_string(&item.payload)?.replace('\'', "''")
        );
        connection.execute(&statement)?;
        last_insert_rowid(&connection)
    } else {
        Err("No database connection available".into())
    }
}

/// Removes items down to the specified item ID in a conversation.
pub fn trim_items(conversation_id: i64, down_to_item_id: i64) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let statement = format!(
            "DELETE FROM aichat_items WHERE conversation_id = {} AND id >= {}",
            conversation_id, down_to_item_id
        );
        connection.execute(&statement)?;
        Ok(())
    } else {
        Err("No database connection available".into())
    }
}

/// Adds a new AI chat conversation with the specified title,
pub fn add_conversation(title: &str) -> Result<i64, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let statement = format!(
            "INSERT INTO aichat_conversations (title) VALUES ('{}')",
            title.replace('\'', "''")
        );
        connection.execute(&statement)?;
        Ok(last_insert_rowid(&connection)?)
    } else {
        Err("No database connection available".into())
    }
}

/// Deletes a conversation and all its associated items.
pub fn delete_conversation(conversation_id: i64) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let statement = format!(
            "DELETE FROM aichat_conversations WHERE id = {}",
            conversation_id
        );
        connection.execute(&statement)?;
        let item_statement = format!(
            "DELETE FROM aichat_items WHERE conversation_id = {}",
            conversation_id
        );
        connection.execute(&item_statement)?;
        Ok(())
    } else {
        Err("No database connection available".into())
    }
}

/// Renames an existing AI chat conversation.
pub fn rename_conversation(conversation_id: i64, new_title: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let statement = format!(
            "UPDATE aichat_conversations SET title = '{}' WHERE id = {}",
            new_title.replace('\'', "''"),
            conversation_id
        );
        connection.execute(&statement)?;
        Ok(())
    } else {
        Err("No database connection available".into())
    }
}

/// Retrieves information about an AI chat conversation by its ID.
pub fn get_conversation(conversation_id: i64) -> Result<SqlAiConversation, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let statement = format!(
            "SELECT id, title FROM aichat_conversations WHERE id = {}",
            conversation_id
        );
        let mut cursor = connection.prepare(&statement)?;
        if cursor.next()? == sqlite::State::Row {
            let conversation = SqlAiConversation {
                id: cursor.read::<i64, _>(0)?,
                title: cursor.read::<String, _>(1)?,
            };
            return Ok(conversation);
        }
    }

    Err("Conversation not found".into())
}

/// Retrieves all AI chat conversations.
pub fn get_all_conversations() -> Result<Vec<SqlAiConversation>, Box<dyn std::error::Error>> {
    let mut conversations = Vec::new();
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let mut cursor = connection.prepare("SELECT id, title FROM aichat_conversations ORDER BY created_at ASC")?;
        while cursor.next()? == sqlite::State::Row {
            let conversation = SqlAiConversation {
                id: cursor.read::<i64, _>(0)?,
                title: cursor.read::<String, _>(1)?,
            };
            conversations.push(conversation);
        }
        Ok(conversations)
    } else {
        Err("No database connection available".into())
    }
}

/// Retrieves items for the specified AI chat conversation.
pub fn get_items(conversation_id: i64) -> Result<Vec<SqlAiConversationItem>, Box<dyn std::error::Error>> {
    let mut items = Vec::new();
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let statement = format!(
            "SELECT id, conversation_id, timestamp, payload \
             FROM aichat_items WHERE conversation_id = {} ORDER BY timestamp ASC",
            conversation_id
        );
        let mut cursor = connection.prepare(&statement)?;
        while cursor.next()? == sqlite::State::Row {
            items.push(SqlAiConversationItem {
                id: cursor.read::<i64, _>(0)?,
                conversation_id: cursor.read::<i64, _>(1)?,
                timestamp: cursor.read::<Option<String>, _>(2)?,
                payload: serde_json::from_str(&cursor.read::<String, _>(3)?)?,
            });
        }
        Ok(items)
    } else {
        Err("No database connection available".into())
    }
}

/// Gets the length of user & assistant messages in a conversation.
pub fn get_messages_length(conversation_id: i64) -> Result<usize, Box<dyn std::error::Error>> {
    Ok(get_items(conversation_id)?
        .into_iter()
        //.filter(|msg| matches!(msg.role.as_str(), "user" | "assistant"))
        .filter(|item| matches!(&item.payload,
            SqlAiConversationItemPayload::Message { role, .. }
            if role == "user" || role == "assistant"
        ))
        .count())
}