use crate::SQL_CONNECTION;

/// Gets the top commands sorted by runs descending.
pub fn get_top_commands(limit: i32) -> Result<Vec<(String, i64)>, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let mut statement = connection.prepare("SELECT command, runs FROM desktop_runs ORDER BY runs DESC LIMIT ?1")?;
        let commands = statement.query_map([limit], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(commands)
    } else {
        Err("No database connection available".into())
    }
}

/// Gets the most recently run commands sorted from most recent to least recent.
pub fn get_recent_commands(limit: i32) -> Result<Vec<(String, i64)>, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let mut statement = connection.prepare("SELECT command, runs FROM desktop_runs ORDER BY last_run DESC LIMIT ?1")?;
        let commands = statement.query_map([limit], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(commands)
    } else {
        Err("No database connection available".into())
    }
}

/// Fetches the number of runs for a given command.
pub fn get_runs(command: &str) -> Result<i64, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        match connection.query_row("SELECT runs FROM desktop_runs WHERE command = ?1", [command], |row| row.get(0)) {
            Ok(row) => Ok(row),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Err(format!("No runs found for command: {}", command).into())
            },
            Err(e) => Err(e.into()),
        }
    } else {
        Err("No database connection available".into())
    }
}

/// Increments the run count for a given command, or inserts it if it doesn't exist.
pub fn increment_runs(command: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        connection.lock()?.execute(
            "INSERT INTO desktop_runs (command, runs) VALUES (?1, 1) 
             ON CONFLICT(command) DO UPDATE SET 
                command = excluded.command,
                runs = runs + 1, 
                last_run = CURRENT_TIMESTAMP",
            [command],
        )?;
        Ok(())
    } else {
        Err("No database connection available".into())
    }
}