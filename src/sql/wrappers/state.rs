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

pub async fn get_source_language() -> anyhow::Result<String> {
    SQL_ACTOR.with(|connection| {
        let row: String = connection.query_row("SELECT source_lang_code FROM state WHERE id = 1", [], |row| row.get(0))?;
        Ok(row)
    }).await?
}

pub async fn set_source_language(lang_code: &str) -> anyhow::Result<()> {
    SQL_ACTOR.with({
        let lang_code = lang_code.to_owned();
        move |connection| {
            connection.execute("UPDATE state SET source_lang_code = ?1 WHERE id = 1", [lang_code])?;
            Ok(())
        }
    }).await?
}

pub async fn get_target_language() -> anyhow::Result<String> {
    SQL_ACTOR.with(|connection| {
        let row: String = connection.query_row("SELECT target_lang_code FROM state WHERE id = 1", [], |row| row.get(0))?;
        Ok(row)
    }).await?
}

pub async fn set_target_language(lang_code: &str) -> anyhow::Result<()> {
    SQL_ACTOR.with({
        let lang_code = lang_code.to_owned();
        move |connection| {
            connection.execute("UPDATE state SET target_lang_code = ?1 WHERE id = 1", [lang_code])?;
            Ok(())
        }
    }).await?
}
