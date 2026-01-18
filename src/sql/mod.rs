pub mod actor;
pub mod wrappers;

use crate::SQL_ACTOR;

pub async fn init_database() {
    let _ = SQL_ACTOR.with(|connection| {
        // Create tables if they do not exist
        connection.execute_batch("
            CREATE TABLE IF NOT EXISTS state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                do_not_disturb INTEGER NOT NULL DEFAULT 0
            )
            
            INSERT OR IGNORE INTO state (id, do_not_disturb) 
            VALUES (1, 0)
            
            CREATE TABLE IF NOT EXISTS desktop_runs (
                command TEXT PRIMARY KEY,
                runs INTEGER NOT NULL DEFAULT 0,
                last_run TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            
            CREATE TABLE IF NOT EXISTS timers (
                started_at TIMESTAMP NOT NULL,
                duration INTEGER NOT NULL,
                description TEXT
            )
            
            CREATE TABLE IF NOT EXISTS weather_forecast (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                fetched_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                payload TEXT NOT NULL
            )
            
            CREATE TABLE IF NOT EXISTS weather_alerts (
                id TEXT PRIMARY KEY,
                fetched_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                payload TEXT NOT NULL
            )
            
            CREATE TABLE IF NOT EXISTS aichat_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                current_conversation_id INTEGER,
                last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(current_conversation_id) REFERENCES aichat_conversations(id) ON DELETE SET NULL
            )
            
            INSERT OR IGNORE INTO aichat_state (id, current_conversation_id) 
            VALUES (1, NULL)
            
            CREATE TABLE IF NOT EXISTS aichat_conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            
            CREATE TABLE IF NOT EXISTS aichat_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id INTEGER NOT NULL,
                timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                payload TEXT NOT NULL,
                FOREIGN KEY(conversation_id) REFERENCES aichat_conversations(id) ON DELETE CASCADE
            )
        ")
    }).await.expect("Failed to initialize database");
}