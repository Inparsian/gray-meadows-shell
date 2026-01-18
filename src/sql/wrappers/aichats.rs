use crate::SQL_ACTOR;
use crate::singletons::ai::types::{AiConversation, AiConversationItem, AiConversationItemPayload};

/// Gets the current conversation ID stored in the AI chat state.
pub async fn get_state_conversation_id() -> anyhow::Result<Option<i64>> {
    SQL_ACTOR.with(|connection| {
        let id: Option<i64> = connection.query_row("SELECT current_conversation_id FROM aichat_state WHERE id = 1", [], |row| row.get(0))?;
        Ok(id)
    }).await?
}

/// Sets the current conversation ID in the AI chat state.
pub async fn set_state_conversation_id(conversation_id: Option<i64>) -> anyhow::Result<()> {
    SQL_ACTOR.with(move |connection| {
        connection.execute(
            "UPDATE aichat_state SET current_conversation_id = ?1 WHERE id = 1",
            [conversation_id],
        )?;
        Ok(())
    }).await?
}

/// Ensures there is at least one AI chat conversation in the database.
pub async fn ensure_default_conversation() -> anyhow::Result<()> {
    SQL_ACTOR.with(|connection| {
        let count: i64 = connection.query_row("SELECT COUNT(*) FROM aichat_conversations", [], |row| row.get(0))?;
        if count == 0 {
            connection.execute(
                "INSERT INTO aichat_conversations (title) VALUES (?1)",
                ["Default Conversation"],
            )?;
        }
        
        Ok(())
    }).await?
}

/// Adds an item to the specified AI chat conversation and returns its new ID.
pub async fn add_item(item: &AiConversationItem) -> anyhow::Result<i64> {
    SQL_ACTOR.with({
        let conversation_id = item.conversation_id;
        let payload = serde_json::to_string(&item.payload)?;
        move |connection| {
            connection.execute(
                "INSERT INTO aichat_items (conversation_id, payload) VALUES (?1, ?2)",
                (conversation_id, payload),
            )?;
            Ok(connection.last_insert_rowid())
        }
    }).await?
}

/// Removes items down to the specified item ID in a conversation.
pub async fn trim_items(conversation_id: i64, down_to_item_id: i64) -> anyhow::Result<()> {
    SQL_ACTOR.with(move |connection| {
        connection.execute(
            "DELETE FROM aichat_items WHERE conversation_id = ?1 AND id >= ?2",
            (conversation_id, down_to_item_id)
        )?;
        Ok(())
    }).await?
}

/// Adds a new AI chat conversation with the specified title,
pub async fn add_conversation(title: &str) -> anyhow::Result<i64> {
    SQL_ACTOR.with({
        let title = title.to_owned();
        move |connection| {
            connection.execute(
                "INSERT INTO aichat_conversations (title) VALUES (?1)",
                [title.as_str()],
            )?;
            Ok(connection.last_insert_rowid())
        }
    }).await?
}

/// Deletes a conversation and all its associated items.
pub async fn delete_conversation(conversation_id: i64) -> anyhow::Result<()> {
    SQL_ACTOR.with(move |connection| {
        connection.execute("DELETE FROM aichat_conversations WHERE id = ?1", [conversation_id])?;
        connection.execute("DELETE FROM aichat_items WHERE conversation_id = ?1", [conversation_id])?;
        Ok(())
    }).await?
}

/// Renames an existing AI chat conversation.
pub async fn rename_conversation(conversation_id: i64, new_title: &str) -> anyhow::Result<()> {
    SQL_ACTOR.with({
        let new_title = new_title.to_owned();
        move |connection| {
            connection.execute(
                "UPDATE aichat_conversations SET title = ?1 WHERE id = ?2",
                (&new_title, conversation_id)
            )?;
            Ok(())
        }
    }).await?
}

/// Retrieves information about an AI chat conversation by its ID.
pub async fn get_conversation(conversation_id: i64) -> anyhow::Result<AiConversation> {
    SQL_ACTOR.with(move |connection| {
        connection.query_row(
            "SELECT id, title FROM aichat_conversations WHERE id = ?1", [conversation_id],
            |row| Ok(AiConversation {
                id: row.get(0)?,
                title: row.get(1)?
            })
        ).map_err(|e| e.into())
    }).await?
}

/// Retrieves all AI chat conversations.
pub async fn get_all_conversations() -> anyhow::Result<Vec<AiConversation>> {
    SQL_ACTOR.with(|connection| {
        let mut statement = connection.prepare("SELECT id, title FROM aichat_conversations ORDER BY created_at ASC")?;
        let conversations = statement.query_map([], |row| Ok(AiConversation {
            id: row.get(0)?,
            title: row.get(1)?
        }))?.collect::<Result<Vec<_>, _>>()?;
        Ok(conversations)
    }).await?
}

/// Retrieves items for the specified AI chat conversation.
pub async fn get_items(conversation_id: i64) -> anyhow::Result<Vec<AiConversationItem>> {
    SQL_ACTOR.with(move |connection| {
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
    }).await?
}

/// Gets every single image item UUID that's stored in AI chat conversations.
pub async fn get_all_image_item_uuids() -> anyhow::Result<Vec<String>> {
    SQL_ACTOR.with(|connection| {
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
    }).await?
}

/// Gets the length of user & assistant messages in a conversation.
pub async fn get_messages_length(conversation_id: i64) -> anyhow::Result<usize> {
    Ok(get_items(conversation_id).await?
        .into_iter()
        .filter(|item| matches!(&item.payload,
            AiConversationItemPayload::Message { role, .. }
            if role == "user" || role == "assistant"
        ))
        .count())
}