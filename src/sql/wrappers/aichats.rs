use crate::SQL_CONNECTION;
use crate::singletons::ai::types::{AiConversation, AiConversationItem, AiConversationItemPayload};

/// Gets the current conversation ID stored in the AI chat state.
pub fn get_state_conversation_id() -> Result<Option<i64>, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        match connection.query_row("SELECT current_conversation_id FROM aichat_state WHERE id = 1", [], |row| row.get(0)) {
            Ok(row) => Ok(row),
            Err(e) => Err(e.into()),
        }
    } else {
        Err("No database connection available".into())
    }
}

/// Sets the current conversation ID in the AI chat state.
pub fn set_state_conversation_id(conversation_id: Option<i64>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        connection.lock()?.execute(
            "UPDATE aichat_state SET current_conversation_id = ?1 WHERE id = 1",
            [conversation_id],
        )?;
        Ok(())
    } else {
        Err("No database connection available".into())
    }
}

/// Ensures there is at least one AI chat conversation in the database.
pub fn ensure_default_conversation() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let count: i64 = connection.query_row("SELECT COUNT(*) FROM aichat_conversations", [], |row| row.get(0))?;
        if count == 0 {
            connection.execute(
                "INSERT INTO aichat_conversations (title) VALUES (?1)",
                ["Default Conversation"],
            )?;
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
        connection.execute(
            "INSERT INTO aichat_items (conversation_id, payload) VALUES (?1, ?2)",
            (item.conversation_id, serde_json::to_string(&item.payload)?),
        )?;
        Ok(connection.last_insert_rowid())
    } else {
        Err("No database connection available".into())
    }
}

/// Removes items down to the specified item ID in a conversation.
pub fn trim_items(conversation_id: i64, down_to_item_id: i64) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        connection.lock()?.execute(
            "DELETE FROM aichat_items WHERE conversation_id = ?1 AND id >= ?2",
            (conversation_id, down_to_item_id)
        )?;
        Ok(())
    } else {
        Err("No database connection available".into())
    }
}

/// Adds a new AI chat conversation with the specified title,
pub fn add_conversation(title: &str) -> Result<i64, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        connection.execute(
            "INSERT INTO aichat_conversations (title) VALUES (?1)",
            [title],
        )?;
        Ok(connection.last_insert_rowid())
    } else {
        Err("No database connection available".into())
    }
}

/// Deletes a conversation and all its associated items.
pub fn delete_conversation(conversation_id: i64) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        connection.execute("DELETE FROM aichat_conversations WHERE id = ?1", [conversation_id])?;
        connection.execute("DELETE FROM aichat_items WHERE conversation_id = ?1", [conversation_id])?;
        Ok(())
    } else {
        Err("No database connection available".into())
    }
}

/// Renames an existing AI chat conversation.
pub fn rename_conversation(conversation_id: i64, new_title: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        connection.lock()?.execute(
            "UPDATE aichat_conversations SET title = ?1 WHERE id = ?2",
            (new_title, conversation_id)
        )?;
        Ok(())
    } else {
        Err("No database connection available".into())
    }
}

/// Retrieves information about an AI chat conversation by its ID.
pub fn get_conversation(conversation_id: i64) -> Result<AiConversation, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        match connection.query_row(
            "SELECT id, title FROM aichat_conversations WHERE id = ?1", [conversation_id],
            |row| Ok(AiConversation {
                id: row.get(0)?,
                title: row.get(1)?
            })
        ) {
            Ok(row) => Ok(row),
            Err(e) => Err(e.into()),
        }
    } else {
        Err("Conversation not found".into())
    }
}

/// Retrieves all AI chat conversations.
pub fn get_all_conversations() -> Result<Vec<AiConversation>, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let mut statement = connection.prepare("SELECT id, title FROM aichat_conversations ORDER BY created_at ASC")?;
        let conversations = statement.query_map([], |row| Ok(AiConversation {
            id: row.get(0)?,
            title: row.get(1)?
        }))?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(conversations)
    } else {
        Err("No database connection available".into())
    }
}

/// Retrieves items for the specified AI chat conversation.
pub fn get_items(conversation_id: i64) -> Result<Vec<AiConversationItem>, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let mut statement = connection.prepare("SELECT id, conversation_id, timestamp, payload \
         FROM aichat_items WHERE conversation_id = ?1 ORDER BY timestamp ASC")?;
        let items = statement.query_map([conversation_id], |row| Ok(AiConversationItem {
            id: row.get(0)?,
            conversation_id: row.get(1)?,
            payload: serde_json::from_value(row.get::<_,serde_json::Value>(3)?)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            timestamp: row.get(2)?
        }))?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(items)
    } else {
        Err("No database connection available".into())
    }
}

/// Gets every single image item UUID that's stored in AI chat conversations.
pub fn get_all_image_item_uuids() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let mut statement = connection.prepare("SELECT payload FROM aichat_items WHERE json_extract(payload, '$.type') = 'image'")?;
        let uuids = statement.query_map([], |row| {
            let payload = serde_json::from_value(row.get::<_,serde_json::Value>(0)?)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            Ok(payload)
        })?
        .filter_map(|payload| match payload {
            Ok(AiConversationItemPayload::Image { uuid }) => Some(Ok(uuid)),
            Ok(_) => None,
            Err(err) => Some(Err(err))
        })
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(uuids)
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