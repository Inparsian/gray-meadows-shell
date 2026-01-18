use chrono::NaiveDateTime;

use crate::SQL_ACTOR;
use crate::singletons::weather::schemas::openmeteo::OpenMeteoResponse;

pub async fn get_weather_forecast() -> anyhow::Result<(chrono::NaiveDateTime, OpenMeteoResponse)> {
    SQL_ACTOR.with(|connection| {
        let row = connection.query_row(
            "SELECT fetched_at, payload FROM weather_forecast WHERE id = 1", [],
            |row| {
                let fetched_at = NaiveDateTime::parse_from_str(&row.get::<_,String>(0)?, "%Y-%m-%d %H:%M:%S")
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                let payload: OpenMeteoResponse = serde_json::from_value(row.get::<_,serde_json::Value>(1)?)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                Ok((fetched_at, payload))
            }
        )?;
        
        Ok(row)
    }).await?
}

pub async fn set_weather_forecast(forecast: &OpenMeteoResponse) -> anyhow::Result<()> {
    SQL_ACTOR.with({
        let serialized = serde_json::to_string(forecast)?;
        move |connection| {
            connection.execute(
                "INSERT INTO weather_forecast (id, payload) VALUES (1, ?1) \
                ON CONFLICT(id) DO UPDATE SET payload = excluded.payload, fetched_at = CURRENT_TIMESTAMP",
                [serialized]
            )?;
            Ok(())
        }
    }).await?
}