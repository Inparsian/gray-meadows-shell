use crate::SQL_CONNECTION;

pub fn get_do_not_disturb() -> Result<bool, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        match connection.query_row("SELECT do_not_disturb FROM state WHERE id = 1", [], |row| row.get(0)) {
            Ok(row) => Ok(row),
            Err(e) => Err(e.into()),
        }
    } else {
        Err("No database connection available".into())
    }
}

pub fn set_do_not_disturb(dnd: bool) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        connection.lock()?.execute("UPDATE state SET do_not_disturb = ?1 WHERE id = 1", [dnd as i32])?;
        Ok(())
    } else {
        Err("No database connection available".into())
    }
}