use crate::SQL_CONNECTION;

#[allow(dead_code)]
pub struct SqlAiConversationMessage {
    pub id: i64,
    pub conversation_id: i64,
    pub role: String,
    pub content: String,
    pub tool_call_id: Option<String>,
    pub tool_call_function: Option<String>,
    pub tool_call_arguments: Option<String>,
    pub timestamp: String,
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
        return Ok(());
    }
    Err("No database connection available".into())
}

/// Adds a message to the specified AI chat conversation.
pub fn add_message(message: &SqlAiConversationMessage) -> Result<(), Box<dyn std::error::Error>> {
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
        return Ok(());
    }
    Err("No database connection available".into())
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
                timestamp: cursor.read::<String, _>(7)?,
            };
            messages.push(message);
        }
        return Ok(messages);
    }
    Err("No database connection available".into())
}