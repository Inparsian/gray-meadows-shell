mod ai;
mod weather;
mod screen_recorder;

pub use ai::*;
pub use weather::*;
pub use screen_recorder::*;

pub fn deserialize_insensitive<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize as _;
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(serde::de::Error::custom)
}