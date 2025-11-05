use crate::SQL_CONNECTION;

/// Gets the top commands sorted by runs descending.
pub fn get_top_commands(limit: usize) -> Result<Vec<(String, i64)>, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let statement = format!("SELECT command, runs FROM desktop_runs ORDER BY runs DESC LIMIT {}", limit);
        let mut cursor = connection.prepare(&statement)?;
        let mut results = Vec::new();
        while cursor.next()? == sqlite::State::Row {
            let command = cursor.read::<String, _>(0)?;
            let runs = cursor.read::<i64, _>(1)?;
            results.push((command, runs));
        }
        return Ok(results);
    }

    Err("No database connection available".into())
}

/// Gets the most recently run commands sorted from most recent to least recent.
pub fn get_recent_commands(limit: usize) -> Result<Vec<(String, i64)>, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let statement = format!("SELECT command, runs FROM desktop_runs ORDER BY last_run DESC LIMIT {}", limit);
        let mut cursor = connection.prepare(&statement)?;
        let mut results = Vec::new();
        while cursor.next()? == sqlite::State::Row {
            let command = cursor.read::<String, _>(0)?;
            let runs = cursor.read::<i64, _>(1)?;
            results.push((command, runs));
        }
        return Ok(results);
    }

    Err("No database connection available".into())
}

/// Fetches the number of runs for a given command.
pub fn get_runs(command: &str) -> Result<i64, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let statement = format!("SELECT * FROM desktop_runs WHERE command = '{}'", command.replace('\'', "''"));
        let mut cursor = connection.prepare(&statement)?;
        return if cursor.next()? == sqlite::State::Row {
            let value = cursor.read::<i64, _>(1)?;
            Ok(value)
        } else {
            Err(Box::new(sqlite::Error {
                code: Some(1),
                message: Some(format!("No runs found for command: {}", command)),
            }))
        }
    }

    Err("No database connection available".into())
}

/// Increments the run count for a given command, or inserts it if it doesn't exist.
pub fn increment_runs(command: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let runs = get_runs(command).unwrap_or(0);
        let connection = connection.lock()?;
        let statement = if runs > 0 {
            format!("
                UPDATE desktop_runs SET
                    runs = {},
                    last_run = CURRENT_TIMESTAMP
                WHERE command = '{}'
            ", runs + 1, command.replace('\'', "''"))
        } else {
            format!("INSERT INTO desktop_runs (command, runs) VALUES ('{}', 1)", command.replace('\'', "''"))
        };

        connection.execute(&statement)?;
        return Ok(());
    }

    Err("No database connection available".into())
}