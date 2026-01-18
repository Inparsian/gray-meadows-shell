use chrono::NaiveDateTime;

use crate::SQL_CONNECTION;
use crate::singletons::weather::schemas::openmeteo::OpenMeteoResponse;

pub fn get_weather_forecast() -> Result<(chrono::NaiveDateTime, OpenMeteoResponse), Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        match connection.query_row(
            "SELECT fetched_at, payload FROM weather_forecast WHERE id = 1", [],
            |row| {
                let fetched_at = NaiveDateTime::parse_from_str(&row.get::<_,String>(0)?, "%Y-%m-%d %H:%M:%S")
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                let payload = serde_json::from_value(row.get::<_,serde_json::Value>(1)?)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                Ok((fetched_at, payload))
            }
        ) {
            Ok(row) => Ok(row),
            Err(e) => Err(e.into()),
        }
    } else {
        Err("No database connection available".into())
    }
}

pub fn set_weather_forecast(forecast: &OpenMeteoResponse) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let serialized = serde_json::to_string(forecast)?;
        connection.lock()?.execute(
            "INSERT INTO weather_forecast (id, payload) VALUES (1, ?1) \
            ON CONFLICT(id) DO UPDATE SET payload = excluded.payload, fetched_at = CURRENT_TIMESTAMP",
            [serialized]
        )?;
        Ok(())
    } else {
        Err("No database connection available".into())
    }
}