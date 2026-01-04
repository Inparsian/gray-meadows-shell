pub mod wrappers;

use sqlite::Connection;

pub fn establish_connection() -> Result<Connection, Box<dyn std::error::Error>> {
    let config_dir = crate::filesystem::get_config_directory();
    let db_path = format!("{}/sqlite.db", config_dir);

    if !std::path::Path::new(&config_dir).exists() {
        std::fs::create_dir_all(&config_dir)?;
    }

    let connection = Connection::open(db_path)?;

    // Create tables if they do not exist
    connection.execute("
        CREATE TABLE IF NOT EXISTS desktop_runs (
            command TEXT NOT NULL,
            runs INTEGER NOT NULL DEFAULT 0,
            last_run TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    ")?;

    connection.execute("
        CREATE TABLE IF NOT EXISTS timers (
            started_at TIMESTAMP NOT NULL,
            duration INTEGER NOT NULL,
            description TEXT
        )
    ")?;

    // Store the JSON payloads, as they contain a lot of nested data returned
    // by the OpenMateo and NWS APIs.
    // The weather_forecast table is restricted to a single row (id = 1) to always store only the latest forecast.
    connection.execute("
        CREATE TABLE IF NOT EXISTS weather_forecast (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            fetched_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            payload TEXT NOT NULL
        )
    ")?;

    connection.execute("
        CREATE TABLE IF NOT EXISTS weather_alerts (
            id TEXT PRIMARY KEY,
            fetched_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            payload TEXT NOT NULL
        )
    ")?;

    // Tables for AI chat conversations
    connection.execute("
        CREATE TABLE IF NOT EXISTS aichat_conversations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
    ")?;

    // Items will be a catch-all for messages, reasoning, function calls, etc.
    // The Responses API is very different from the Chat Completions API and introduces
    // a bajillion item types, so JSON payloads are stored inside which will be
    // serialized and deserialized as needed.
    connection.execute("
        CREATE TABLE IF NOT EXISTS aichat_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            conversation_id INTEGER NOT NULL,
            timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            payload TEXT NOT NULL,
            FOREIGN KEY(conversation_id) REFERENCES aichat_conversations(id) ON DELETE CASCADE
        )
    ")?;

    Ok(connection)
}

pub fn last_insert_rowid(connection: &Connection) -> Result<i64, Box<dyn std::error::Error>> {
    let mut cursor = connection.prepare("SELECT last_insert_rowid()")?;
    let rowid = if cursor.next()? == sqlite::State::Row {
        cursor.read::<i64, _>(0)?
    } else {
        0
    };
    Ok(rowid)
}