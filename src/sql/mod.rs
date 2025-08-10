use sqlite::Connection;
use std::sync::Mutex;

pub struct SqliteWrapper {
    connection: Mutex<Connection>
}

impl std::fmt::Debug for SqliteWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteWrapper")
            .field("connection", &"<Connection>")
            .finish()
    }
}

impl SqliteWrapper {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let connection = establish_connection()?;

        Ok(Self {
            connection: Mutex::new(connection)
        })
    }

    /// Fetches the number of runs for a given command.
    #[allow(dead_code)]
    pub fn get_runs(&self, command: &str) -> Result<i64, sqlite::Error> {
        let connection = self.connection.lock().unwrap();
        let statement = format!("SELECT runs FROM desktop_runs WHERE command = '{}'", command.replace('\'', "''"));

        let mut cursor = connection.prepare(&statement)?;
        if cursor.next()? == sqlite::State::Row {
            let value = cursor.read::<i64, _>(1)?;
            return Ok(value);
        }

        Err(sqlite::Error {
            code: Some(1),
            message: Some(format!("No runs found for command: {}", command)),
        })
    }

    /// Increments the run count for a given command, or inserts it if it doesn't exist.
    #[allow(dead_code)]
    pub fn increment_runs(&self, command: &str) -> Result<(), sqlite::Error> {
        let connection = self.connection.lock().unwrap();
        let runs = self.get_runs(command).unwrap_or(0);
        let statement = if runs > 0 {
            format!("UPDATE desktop_runs SET runs = {} WHERE command = '{}'", runs + 1, command.replace('\'', "''"))
        } else {
            format!("INSERT INTO desktop_runs (command, runs) VALUES ('{}', 1)", command.replace('\'', "''"))
        };

        connection.execute(&statement)?;
        Ok(())
    }
}

pub fn establish_connection() -> Result<Connection, Box<dyn std::error::Error>> {
    let config_dir = crate::helpers::filesystem::get_config_directory();
    let db_path = format!("{}/sqlite.db", config_dir);

    if !std::path::Path::new(&config_dir).exists() {
        std::fs::create_dir_all(&config_dir)?;
    }

    let connection = Connection::open(db_path)?;

    // Create tables if they do not exist
    connection.execute("
        CREATE TABLE IF NOT EXISTS desktop_runs (
            command TEXT NOT NULL,
            runs INTEGER NOT NULL DEFAULT 0
        )
    ")?;

    Ok(connection)
}