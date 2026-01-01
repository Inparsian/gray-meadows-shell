use crate::SQL_CONNECTION;
use super::super::last_insert_rowid;

#[derive(Debug, Clone)]
pub struct SqlAiConversation {
    pub id: i64,
    pub title: String,
}

pub struct SqlAiConversationToolCall {
    pub id: String,
    pub function: String,
    pub arguments: String,
}

pub struct SqlAiConversationMessage {
    pub id: i64,
    pub conversation_id: i64,
    pub role: String,
    pub content: String,
    pub tool_calls: Vec<SqlAiConversationToolCall>,
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

/// Adds a message to the specified AI chat conversation and returns its new ID.
pub fn add_message(message: &SqlAiConversationMessage) -> Result<i64, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let statement = format!(
            "INSERT INTO aichat_messages (conversation_id, role, content) \
             VALUES ({}, '{}', '{}')",
            message.conversation_id,
            message.role.replace('\'', "''"),
            message.content.replace('\'', "''"),
        );
        connection.execute(&statement)?;
        let message_id = last_insert_rowid(&connection)?;

        // Insert tool calls if any
        for tool_call in &message.tool_calls {
            let tool_statement = format!(
                "INSERT INTO aichat_tool_calls (tool_id, message_id, function, arguments) \
                 VALUES ('{}', {}, '{}', '{}')",
                tool_call.id.replace('\'', "''"),
                message_id,
                tool_call.function.replace('\'', "''"),
                tool_call.arguments.replace('\'', "''"),
            );
            connection.execute(&tool_statement)?;
        }

        Ok(message_id)
    } else {
        Err("No database connection available".into())
    }
}

/// Removes messages down to the specified message ID in a conversation.
pub fn trim_messages(conversation_id: i64, down_to_message_id: i64) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let statement = format!(
            "DELETE FROM aichat_messages WHERE conversation_id = {} AND id >= {}",
            conversation_id, down_to_message_id
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

/// Deletes a conversation and all its associated messages.
pub fn delete_conversation(conversation_id: i64) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let statement = format!(
            "DELETE FROM aichat_conversations WHERE id = {}",
            conversation_id
        );
        connection.execute(&statement)?;
        let message_statement = format!(
            "DELETE FROM aichat_messages WHERE conversation_id = {}",
            conversation_id
        );
        connection.execute(&message_statement)?;
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

/// Gets all tool calls associated with a message.
fn get_tool_calls_for_message(
    message_id: i64,
    connection: &sqlite::Connection,
) -> Result<Vec<SqlAiConversationToolCall>, Box<dyn std::error::Error>> {
    let mut tool_calls = Vec::new();
    let statement = format!(
        "SELECT tool_id, function, arguments FROM aichat_tool_calls WHERE message_id = {}",
        message_id
    );
    let mut cursor = connection.prepare(&statement)?;
    while cursor.next()? == sqlite::State::Row {
        let tool_call = SqlAiConversationToolCall {
            id: cursor.read::<String, _>(0)?,
            function: cursor.read::<String, _>(1)?,
            arguments: cursor.read::<String, _>(2)?,
        };
        tool_calls.push(tool_call);
    }
    Ok(tool_calls)
}

/// Retrieves messages for the specified AI chat conversation.
pub fn get_messages(conversation_id: i64) -> Result<Vec<SqlAiConversationMessage>, Box<dyn std::error::Error>> {
    let mut messages = Vec::new();
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let statement = format!(
            "SELECT id, conversation_id, role, content, timestamp \
             FROM aichat_messages WHERE conversation_id = {} ORDER BY timestamp ASC",
            conversation_id
        );
        let mut cursor = connection.prepare(&statement)?;
        while cursor.next()? == sqlite::State::Row {
            let id = cursor.read::<i64, _>(0)?;
            let tool_calls = get_tool_calls_for_message(id, &connection)?;
            let message = SqlAiConversationMessage {
                id,
                conversation_id: cursor.read::<i64, _>(1)?,
                role: cursor.read::<String, _>(2)?,
                content: cursor.read::<String, _>(3)?,
                tool_calls,
                timestamp: cursor.read::<Option<String>, _>(4)?,
            };
            messages.push(message);
        }
        Ok(messages)
    } else {
        Err("No database connection available".into())
    }
}

/// Gets the length of messages in a conversation.
pub fn get_messages_length(conversation_id: i64) -> Result<usize, Box<dyn std::error::Error>> {
    Ok(get_messages(conversation_id)?
        .into_iter()
        .filter(|msg| matches!(msg.role.as_str(), "user" | "assistant"))
        .count())
}