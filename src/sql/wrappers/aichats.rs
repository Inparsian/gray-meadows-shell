use crate::SQL_CONNECTION;
use super::super::last_insert_rowid;

#[derive(Debug, Clone)]
pub struct SqlAiConversation {
    pub id: i64,
    pub title: String,
}

pub struct SqlAiConversationMessage {
    pub id: i64,
    pub conversation_id: i64,
    pub role: String,
    pub content: String,
    pub tool_call_id: Option<String>,
    pub tool_call_function: Option<String>,
    pub tool_call_arguments: Option<String>,
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

                // Insert our prompt
                let conversation_id = last_insert_rowid(&connection)?;
                insert_system_prompt_with_connection(conversation_id, &connection)?;
            }
        }
        Ok(())
    } else {
        Err("No database connection available".into())
    }
}

/// Inserts the default system prompt message into a conversation with the specified connection.
fn insert_system_prompt_with_connection(conversation_id: i64, connection: &sqlite::Connection) -> Result<(), Box<dyn std::error::Error>> {
    let prompt = crate::APP.config.ai.prompt.clone();
    let statement = format!(
        "INSERT INTO aichat_messages (conversation_id, role, content) \
         VALUES ({}, 'system', '{}')",
        conversation_id,
        prompt.replace('\'', "''")
    );
    connection.execute(&statement)?;
    Ok(())
}

pub fn insert_system_prompt(conversation_id: i64) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        insert_system_prompt_with_connection(conversation_id, &connection)
    } else {
        Err("No database connection available".into())
    }
}

/// Adds a message to the specified AI chat conversation and returns its new ID.
pub fn add_message(message: &SqlAiConversationMessage) -> Result<i64, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let statement = format!(
            "INSERT INTO aichat_messages (conversation_id, role, content, tool_call_id, tool_call_function, tool_call_arguments) \
             VALUES ({}, '{}', '{}', {}, {}, {})",
            message.conversation_id,
            message.role.replace('\'', "''"),
            message.content.replace('\'', "''"),
            message.tool_call_id.as_ref().map_or_else(|| "NULL".to_owned(), |id| format!("'{}'", id.replace('\'', "''"))),
            message.tool_call_function.as_ref().map_or_else(|| "NULL".to_owned(), |func| format!("'{}'", func.replace('\'', "''"))),
            message.tool_call_arguments.as_ref().map_or_else(|| "NULL".to_owned(), |args| format!("'{}'", args.replace('\'', "''")))
        );
        connection.execute(&statement)?;
        last_insert_rowid(&connection)
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
/// inserts the system prompt message, and returns its new ID.
pub fn add_conversation(title: &str) -> Result<i64, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let statement = format!(
            "INSERT INTO aichat_conversations (title) VALUES ('{}')",
            title.replace('\'', "''")
        );
        connection.execute(&statement)?;
        let conversation_id = last_insert_rowid(&connection)?;
        insert_system_prompt_with_connection(conversation_id, &connection)?;
        Ok(conversation_id)
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

/// Retrieves messages for the specified AI chat conversation.
pub fn get_messages(conversation_id: i64) -> Result<Vec<SqlAiConversationMessage>, Box<dyn std::error::Error>> {
    let mut messages = Vec::new();
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let statement = format!(
            "SELECT id, conversation_id, role, content, tool_call_id, tool_call_function, tool_call_arguments, timestamp \
             FROM aichat_messages WHERE conversation_id = {} ORDER BY timestamp ASC",
            conversation_id
        );
        let mut cursor = connection.prepare(&statement)?;
        while cursor.next()? == sqlite::State::Row {
            let message = SqlAiConversationMessage {
                id: cursor.read::<i64, _>(0)?,
                conversation_id: cursor.read::<i64, _>(1)?,
                role: cursor.read::<String, _>(2)?,
                content: cursor.read::<String, _>(3)?,
                tool_call_id: cursor.read::<Option<String>, _>(4)?,
                tool_call_function: cursor.read::<Option<String>, _>(5)?,
                tool_call_arguments: cursor.read::<Option<String>, _>(6)?,
            };
            messages.push(message);
        }
        Ok(messages)
    } else {
        Err("No database connection available".into())
    }
}