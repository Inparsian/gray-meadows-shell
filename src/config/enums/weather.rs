use serde::{Deserialize, Serialize};
use strum::{EnumString, Display};

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
#[strum(ascii_case_insensitive)]
pub enum WeatherTemperatureUnit {
    #[strum(to_string = "celsius", serialize = "c")]
    Celsius,
    #[strum(to_string = "fahrenheit", serialize = "f")]
    Fahrenheit,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
#[strum(ascii_case_insensitive)]
pub enum WeatherSpeedUnit {
    #[strum(to_string = "kmh", serialize = "kmph", serialize = "km/h")]
    Kmh,
    #[strum(to_string = "ms", serialize = "m/s")]
    Ms,
    #[strum(to_string = "kn", serialize = "knots")]
    Kn,
    #[strum(to_string = "mph", serialize = "mp/h")]
    Mph,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
#[strum(ascii_case_insensitive)]
pub enum WeatherPrecipitationUnit {
    #[strum(to_string = "mm", serialize = "millimeter", serialize = "millimeters")]
    Mm,
    #[strum(to_string = "inch", serialize = "in", serialize = "inches")]
    Inch,
}