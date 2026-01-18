use crate::SQL_ACTOR;

#[derive(Debug, Clone)]
pub struct DesktopRunsEntry {
    pub command: String,
    pub runs: i64,
    pub last_run: chrono::DateTime<chrono::Local>,
}

/// Fetches all runs for all commands.
pub async fn get_all_runs() -> anyhow::Result<Vec<DesktopRunsEntry>> {
    SQL_ACTOR.with({
        move |connection| {
            let mut statement = connection.prepare("SELECT command, runs, last_run FROM desktop_runs")?;
            let commands = statement.query_map([], |row| {
                Ok(DesktopRunsEntry {
                    command: row.get(0)?,
                    runs: row.get(1)?,
                    last_run: row.get(2)?,
                })
            })?.collect::<Result<Vec<_>, _>>()?;
            Ok(commands)
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
                    last_run = datetime('subsec')",
                [command],
            )?;
            Ok(())
        }
    }).await?
}