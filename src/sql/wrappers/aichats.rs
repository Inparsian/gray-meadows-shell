use crate::SQL_CONNECTION;
use crate::singletons::ai::types::{AiConversation, AiConversationItem, AiConversationItemPayload};
use super::super::last_insert_rowid;

/// Gets the current conversation ID stored in the AI chat state.
pub fn get_state_conversation_id() -> Result<Option<i64>, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let mut cursor = connection.prepare("SELECT current_conversation_id FROM aichat_state WHERE id = 1")?;
        if cursor.next()? == sqlite::State::Row {
            let conversation_id = cursor.read::<Option<i64>, _>(0)?;
            return Ok(conversation_id);
        }
    }

    Err("No database connection available".into())
}

/// Sets the current conversation ID in the AI chat state.
pub fn set_state_conversation_id(conversation_id: Option<i64>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let statement = conversation_id.map_or_else(
            || "UPDATE aichat_state SET current_conversation_id = NULL WHERE id = 1".to_owned(), 
            |id| format!(
                "UPDATE aichat_state SET current_conversation_id = {} WHERE id = 1",
                id
            )
        );
        connection.execute(&statement)?;
        Ok(())
    } else {
        Err("No database connection available".into())
    }
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
pub fn add_item(item: &AiConversationItem) -> Result<i64, Box<dyn std::error::Error>> {
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
pub fn get_conversation(conversation_id: i64) -> Result<AiConversation, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let statement = format!(
            "SELECT id, title FROM aichat_conversations WHERE id = {}",
            conversation_id
        );
        let mut cursor = connection.prepare(&statement)?;
        if cursor.next()? == sqlite::State::Row {
            let conversation = AiConversation {
                id: cursor.read::<i64, _>(0)?,
                title: cursor.read::<String, _>(1)?,
            };
            return Ok(conversation);
        }
    }

    Err("Conversation not found".into())
}

/// Retrieves all AI chat conversations.
pub fn get_all_conversations() -> Result<Vec<AiConversation>, Box<dyn std::error::Error>> {
    let mut conversations = Vec::new();
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let mut cursor = connection.prepare("SELECT id, title FROM aichat_conversations ORDER BY created_at ASC")?;
        while cursor.next()? == sqlite::State::Row {
            let conversation = AiConversation {
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
pub fn get_items(conversation_id: i64) -> Result<Vec<AiConversationItem>, Box<dyn std::error::Error>> {
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
            items.push(AiConversationItem {
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
        .filter(|item| matches!(&item.payload,
            AiConversationItemPayload::Message { role, .. }
            if role == "user" || role == "assistant"
        ))
        .count())
}