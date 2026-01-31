pub mod gemini;

use serde_json::json;

use crate::config::read_config;
use crate::session::SessionAction;
use crate::services::mpris::{self, mpris_player::LoopStatus};
use crate::services::weather::{WEATHER, get_wmo_code, get_daily_at};
use super::types::AiFunction;

pub fn get_tools() -> Vec<AiFunction> {
    let app_config = read_config();
    let mut tools = vec![];

    if app_config.ai.features.mpris_control {
        tools.push(AiFunction {
            name: "control_mpris_player".to_owned(),
            description: "Performs an action on the default MPRIS player such as play, pause, stop, toggle play/pause, or skip tracks.".to_owned(),
            strict: true,
            schema: json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "description": "The action to perform on the MPRIS player",
                        "enum": [
                            "toggle",
                            "play",
                            "pause",
                            "stop",
                            "next",
                            "previous"
                        ]
                    }
                },
                "required": ["action"],
                "additionalProperties": false
            }),
        });

        tools.push(AiFunction {
            name: "set_mpris_loop_state".to_owned(),
            description: "Change or cycle the loop state for the default MPRIS player.".to_owned(),
            strict: false,
            schema: json!({
                "type": "object",
                "properties": {
                    "loop_state": {
                        "type": "string",
                        "description": "Requested loop state for the player. Must be one of 'off', 'playlist', or 'track'. If omitted, cycles to the next state.",
                        "enum": ["off", "playlist", "track"]
                    }
                },
                "required": [],
                "additionalProperties": false
            }),
        });
        
        tools.push(AiFunction {
            name: "set_mpris_shuffle_state".to_owned(),
            description: "Change or toggle the shuffle state for the default MPRIS player.".to_owned(),
            strict: false,
            schema: json!({
                "type": "object",
                "properties": {
                    "shuffle": {
                        "type": "boolean",
                        "description": "If provided, sets shuffle to this value; if omitted, the shuffle state will be toggled."
                    }
                },
                "required": [],
                "additionalProperties": false
            }),
        });
    }

    if app_config.ai.features.power_control {
        tools.push(AiFunction {
            name: "perform_power_action".to_owned(),
            description: "Performs a system power action.".to_owned(),
            strict: true,
            schema: json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "description": "Power action to perform",
                        "enum": [
                            "lock",
                            "logout",
                            "suspend",
                            "hibernate",
                            "reboot",
                            "shutdown"
                        ]
                    }
                },
                "required": [
                    "action"
                ],
                "additionalProperties": false
            }),
        });
    }

    if app_config.weather.enabled && app_config.ai.features.weather_info {
        tools.push(AiFunction {
            name: "get_current_weather".to_owned(),
            description: "Fetches the current weather information from the weather service.".to_owned(),
            strict: false,
            schema: json!({
                "type": "object",
                "properties": {},
                "required": [],
                "additionalProperties": false
            }),
        });
    }

    tools
}

pub fn call_tool(name: &str, args: &str) -> serde_json::Value {
    let success = json!({
        "success": true
    });

    match name {
        "perform_power_action" => {
            match serde_json::from_str::<serde_json::Value>(args) {
                Ok(args) => {
                    args.get("action").and_then(|v| v.as_str()).map_or_else(|| json!({
                        "success": false,
                        "error": "Missing 'action' parameter"
                    }), |action| {
                        match action {
                            "lock" => SessionAction::Lock.run(),
                            "logout" => SessionAction::Logout.run(),
                            "suspend" => SessionAction::Suspend.run(),
                            "hibernate" => SessionAction::Hibernate.run(),
                            "reboot" => SessionAction::Reboot.run(),
                            "shutdown" => SessionAction::Shutdown.run(),

                            _ => return json!({
                                "success": false,
                                "error": format!("Invalid action: {}", action)
                            }),
                        }

                        success
                    })
                }
                
                Err(e) => json!({
                    "success": false,
                    "error": format!("Failed to parse arguments: {}", e)
                }),
            }
        },

        "control_mpris_player" => {
            match serde_json::from_str::<serde_json::Value>(args) {
                Ok(args) => {
                    args.get("action").and_then(|v| v.as_str()).map_or_else(|| json!({
                        "success": false,
                        "error": "Missing 'action' parameter"
                    }), |action| {
                        mpris::with_default_player_mut(|player| {
                            let _ = match action {
                                "toggle" => player.play_pause(),
                                "play" => player.play(),
                                "pause" => player.pause(),
                                "stop" => player.stop(),
                                "next" => player.next(),
                                "previous" => player.previous(),

                                _ => return json!({
                                    "success": false,
                                    "error": format!("Invalid action: {}", action)
                                }),
                            };

                            success
                        }).unwrap_or_else(|| json!({
                            "success": false,
                            "error": "No MPRIS player found"
                        }))
                    })
                }
                
                Err(e) => json!({
                    "success": false,
                    "error": format!("Failed to parse arguments: {}", e)
                }),
            }
        },

        "set_mpris_loop_state" => {
            match serde_json::from_str::<serde_json::Value>(args) {
                Ok(args) => {
                    let loop_state_opt = args.get("loop_state").and_then(|v| v.as_str());

                    mpris::with_default_player_mut(|player| {
                        if let Some(loop_state) = loop_state_opt {
                            let _ = match loop_state {
                                "off" => player.set_loop_status(LoopStatus::None),
                                "playlist" => player.set_loop_status(LoopStatus::Playlist),
                                "track" => player.set_loop_status(LoopStatus::Track),

                                _ => return json!({
                                    "success": false,
                                    "error": format!("Invalid loop state: {}", loop_state)
                                }),
                            };
                        } else {
                            // Cycle to the next state
                            let new_state = match player.loop_status {
                                LoopStatus::None => LoopStatus::Playlist,
                                LoopStatus::Playlist => LoopStatus::Track,
                                LoopStatus::Track => LoopStatus::None,
                            };
                            let _ = player.set_loop_status(new_state);
                        }

                        success
                    }).unwrap_or_else(|| json!({
                        "success": false,
                        "error": "No MPRIS player found"
                    }))
                }
                
                Err(e) => json!({
                    "success": false,
                    "error": format!("Failed to parse arguments: {}", e)
                }),
            }
        },

        "set_mpris_shuffle_state" => {
            match serde_json::from_str::<serde_json::Value>(args) {
                Ok(args) => {
                    let shuffle_opt = args.get("shuffle").and_then(|v| v.as_bool());

                    mpris::with_default_player_mut(|player| {
                        let _ = player.set_shuffle(shuffle_opt.unwrap_or(!player.shuffle));
                        success
                    }).unwrap_or_else(|| json!({
                        "success": false,
                        "error": "No MPRIS player found"
                    }))
                }
                
                Err(e) => json!({
                    "success": false,
                    "error": format!("Failed to parse arguments: {}", e)
                }),
            }
        },

        "get_current_weather" => {
            WEATHER.last_response.get_cloned().map_or_else(|| json!({
                "success": false,
                "error": "Weather information is missing, either the weather service is disabled or the information was not fetched yet."
            }), |weather| {
                // Shrink the JSON structure to save tokens
                let weekly_forecast = weather.daily.time
                    .iter()
                    .enumerate()
                    .filter_map(|(i, time)| {
                        if let Some(daily) = get_daily_at(&weather, i) {
                            let weekday = if i > 0 {
                                chrono::NaiveDate::parse_from_str(time, "%Y-%m-%d")
                                    .map_or_else(|_| "nil".to_owned(), |d| d.format("%A").to_string())
                            } else {
                                "Today".to_owned()
                            };
                            let condition = get_wmo_code(daily.weather_code).map_or_else(|| "Unknown", |code| code.text);
                            let temperature_2m_min = format!("{}{}", daily.temperature_2m_min, weather.daily_units.temperature_2m_min);
                            let temperature_2m_max = format!("{}{}", daily.temperature_2m_max, weather.daily_units.temperature_2m_max);
                            
                            Some(json!({
                                "day": weekday,
                                "condition": condition,
                                "high": temperature_2m_max,
                                "low": temperature_2m_min,
                            }))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                
                let temperature_2m = format!("{}{}", weather.current.temperature_2m, weather.current_units.temperature_2m);
                let relative_humidity_2m = format!("{}{}", weather.current.relative_humidity_2m, weather.current_units.relative_humidity_2m);
                let apparent_temperature = format!("{}{}", weather.current.apparent_temperature, weather.current_units.apparent_temperature);
                let precipitation = format!("{} {}", weather.current.precipitation, weather.current_units.precipitation);
                let rain = format!("{} {}", weather.current.rain, weather.current_units.rain);
                let showers = format!("{} {}", weather.current.showers, weather.current_units.showers);
                let snowfall = format!("{} {}", weather.current.snowfall, weather.current_units.snowfall);
                let condition = get_wmo_code(weather.current.weather_code).map_or_else(|| "Unknown", |code| code.text);
                let cloud_cover = format!("{}{}", weather.current.cloud_cover, weather.current_units.cloud_cover);
                let pressure_msl = format!("{} {}", weather.current.pressure_msl, weather.current_units.pressure_msl);
                let surface_pressure = format!("{} {}", weather.current.surface_pressure, weather.current_units.surface_pressure);
                let wind_speed_10m = format!("{} {}", weather.current.wind_speed_10m, weather.current_units.wind_speed_10m);
                let wind_direction_10m = format!("{}{}", weather.current.wind_direction_10m, weather.current_units.wind_direction_10m);
                let wind_gusts_10m = format!("{} {}", weather.current.wind_gusts_10m, weather.current_units.wind_gusts_10m);
                
                json!({
                    "success": true,
                    "weather": {
                        "weekly": weekly_forecast,
                        "current": {
                            "condition": condition,
                            "temperature": temperature_2m,
                            "feels_like": apparent_temperature,
                            "relative_humidity": relative_humidity_2m,
                            "precipitation": precipitation,
                            "rain": rain,
                            "showers": showers,
                            "snowfall": snowfall,
                            "cloud_cover": cloud_cover,
                            "pressure_msl": pressure_msl,
                            "surface_pressure": surface_pressure,
                            "wind_speed": wind_speed_10m,
                            "wind_direction": wind_direction_10m,
                            "wind_gusts": wind_gusts_10m
                        }
                    }
                })
            })
        },

        _ => json!({
            "success": false,
            "error": format!("Unknown function: {}", name)
        }),
    }
}