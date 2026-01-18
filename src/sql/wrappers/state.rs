use crate::SQL_ACTOR;

pub async fn get_do_not_disturb() -> anyhow::Result<bool> {
    SQL_ACTOR.with(|connection| {
        let row: i64 = connection.query_row("SELECT do_not_disturb FROM state WHERE id = 1", [], |row| row.get(0))?;
        Ok(row > 0)
    }).await?
}

pub async fn set_do_not_disturb(dnd: bool) -> anyhow::Result<()> {
    SQL_ACTOR.with(move |connection| {
        connection.execute("UPDATE state SET do_not_disturb = ?1 WHERE id = 1", [dnd as i32])?;
        Ok(())
    }).await?
}