use crate::SQL_ACTOR;

/// Gets the top commands sorted by runs descending.
pub async fn get_top_commands(limit: i32) -> anyhow::Result<Vec<(String, i64)>> {
    SQL_ACTOR.with(move |connection| {
        let mut statement = connection.prepare("SELECT command, runs FROM desktop_runs ORDER BY runs DESC LIMIT ?1")?;
        let commands = statement.query_map([limit], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(commands)
    }).await?
}

/// Gets the most recently run commands sorted from most recent to least recent.
pub async fn get_recent_commands(limit: i32) -> anyhow::Result<Vec<(String, i64)>> {
    SQL_ACTOR.with(move |connection| {
        let mut statement = connection.prepare("SELECT command, runs FROM desktop_runs ORDER BY last_run DESC LIMIT ?1")?;
        let commands = statement.query_map([limit], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(commands)
    }).await?
}

/// Fetches the number of runs for a given command.
pub async fn get_runs(command: &str) -> anyhow::Result<i64> {
    SQL_ACTOR.with({
        let command = command.to_owned();
        move |connection| {
            match connection.query_row("SELECT runs FROM desktop_runs WHERE command = ?1", [&command], |row| row.get(0)) {
                Ok(row) => Ok(row),
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    Err(anyhow::anyhow!(format!("No runs found for command: {}", command)))
                },
                Err(e) => Err(e.into()),
            }
        }
    }).await?
}

/// Increments the run count for a given command, or inserts it if it doesn't exist.
pub async fn increment_runs(command: &str) -> anyhow::Result<()> {
    SQL_ACTOR.with({
        let command = command.to_owned();
        move |connection| {
            connection.execute(
                "INSERT INTO desktop_runs (command, runs) VALUES (?1, 1) 
                 ON CONFLICT(command) DO UPDATE SET 
                    command = excluded.command,
                    runs = runs + 1, 
                    last_run = CURRENT_TIMESTAMP",
                [command],
            )?;
            Ok(())
        }
    }).await?
}