use crate::SQL_CONNECTION;
use crate::singletons::weather::schemas::openmeteo::OpenMeteoResponse;

pub fn get_weather_forecast() -> Result<Option<(chrono::NaiveDateTime, OpenMeteoResponse)>, Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let statement = "SELECT fetched_at, payload FROM weather_forecast WHERE id = 1";
        let mut cursor = connection.prepare(statement)?;
        if cursor.next()? == sqlite::State::Row {
            let fetched_at_str = cursor.read::<String, _>(0)?;
            let fetched_at = chrono::NaiveDateTime::parse_from_str(&fetched_at_str, "%Y-%m-%d %H:%M:%S")?;
            let payload_str = cursor.read::<String, _>(1)?;
            let payload: OpenMeteoResponse = serde_json::from_str(&payload_str)?;
            return Ok(Some((fetched_at, payload)));
        }
    }

    Err("No database connection available".into())
}

pub fn set_weather_forecast(forecast: &OpenMeteoResponse) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(connection) = SQL_CONNECTION.get() {
        let connection = connection.lock()?;
        let serialized = serde_json::to_string(forecast)?;
        let statement = format!(
            "INSERT INTO weather_forecast (id, payload) VALUES (1, '{}') \
            ON CONFLICT(id) DO UPDATE SET payload = excluded.payload, fetched_at = CURRENT_TIMESTAMP",
            serialized.replace('\'', "''")
        );
        connection.execute(&statement)?;
        return Ok(());
    }

    Err("No database connection available".into())
}