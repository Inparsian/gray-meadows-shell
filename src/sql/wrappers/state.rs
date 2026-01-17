use crate::SQL_CONNECTION;

pub fn get_do_not_disturb() -> Result<bool, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let mut cursor = connection.prepare("SELECT do_not_disturb FROM state WHERE id = 1")?;
        if cursor.next()? == sqlite::State::Row {
            let do_not_disturb = cursor.read::<Option<i64>, _>(0)?;
            return Ok(do_not_disturb.unwrap_or(0) != 0);
        }
    }
    
    Err("No database connection available".into())
}

pub fn set_do_not_disturb(dnd: bool) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let statement = format!("UPDATE state SET do_not_disturb = {} WHERE id = 1", dnd as i32);
        connection.execute(&statement)?;
        return Ok(());
    }
    
    Err("No database connection available".into())
}