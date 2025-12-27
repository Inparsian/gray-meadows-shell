use serde_json::json;
use async_openai::error::OpenAIError;
use async_openai::types::chat::{ChatCompletionTools, ChatCompletionTool, FunctionObjectArgs};

use crate::APP;
use crate::session::SessionAction;
use crate::singletons::mpris;
use crate::singletons::mpris::mpris_player::LoopStatus;

pub fn get_tools() -> Result<Vec<ChatCompletionTools>, OpenAIError> {
    let mut tools = vec![];

    if APP.config.ai.features.mpris_control {
        tools.extend(vec![
            ChatCompletionTools::Function(ChatCompletionTool {
                function: FunctionObjectArgs::default()
                    .name("get_mpris_track_info")
                    .description("Gets track information from the current MPRIS player")
                    .build()?,
            }),
            ChatCompletionTools::Function(ChatCompletionTool {
                function: FunctionObjectArgs::default()
                    .name("control_mpris_player")
                    .description("Performs an action on the default MPRIS player such as play, pause, stop, toggle play/pause, or skip tracks.")
                    .strict(true)
                    .parameters(json!({
                        "type": "object",
                        "properties": {
                            "action": {
                                "type": "string",
                                "description": "The action to perform on the player",
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
                        "required": [
                            "action"
                        ],
                        "additionalProperties": false
                    }))
                    .build()?,
            }),
            ChatCompletionTools::Function(ChatCompletionTool {
                function: FunctionObjectArgs::default()
                    .name("set_mpris_loop_state")
                    .description("Change or cycle the loop state for the default MPRIS player.")
                    .strict(false)
                    .parameters(json!({
                        "type": "object",
                        "properties": {
                            "loop_state": {
                                "type": "string",
                                "description": "Requested loop state for the player. Must be one of 'off', 'playlist', or 'track'. If omitted, cycles to the next state.",
                                "enum": [
                                    "off",
                                    "playlist",
                                    "track"
                                ]
                            }
                        },
                        "required": [],
                        "additionalProperties": false
                    }))
                    .build()?,
            }),
            ChatCompletionTools::Function(ChatCompletionTool {
                function: FunctionObjectArgs::default()
                    .name("set_mpris_shuffle_state")
                    .description("Change or toggle the shuffle state for the default MPRIS player.")
                    .strict(false)
                    .parameters(json!({
                        "type": "object",
                        "properties": {
                            "shuffle": {
                                "type": "boolean",
                                "description": "If provided, sets shuffle to this value; if omitted, the shuffle state will be toggled."
                            }
                        },
                        "required": [],
                        "additionalProperties": false
                    }))
                    .build()?,
            }),
        ]);
    }

    if APP.config.ai.features.power_control {
        tools.push(ChatCompletionTools::Function(ChatCompletionTool {
            function: FunctionObjectArgs::default()
                .name("perform_power_action")
                .description("Performs a system power action.")
                .strict(true)
                .parameters(json!({
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
                }))
                .build()?,
        }));
    }
        
    if APP.config.ai.features.wayland_info {
        tools.push(ChatCompletionTools::Function(ChatCompletionTool {
            function: FunctionObjectArgs::default()
                .name("get_focused_wayland_window")
                .description("Gets the currently focused Wayland window along with the current workspace.")
                .build()?,
        }));
    }

    if APP.config.ai.features.current_date_time {
        tools.push(ChatCompletionTools::Function(ChatCompletionTool {
            function: FunctionObjectArgs::default()
                .name("get_current_datetime")
                .description("Get the current date and time")
                .build()?,
        }));
    }

    Ok(tools)
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

        "get_mpris_track_info" => {
            let Some(player) = mpris::get_default_player() else {
                return json!({
                    "success": false,
                    "error": "No MPRIS player found"
                });
            };

            json!({
                "success": true,
                "track": {
                    "title": player.metadata.title.unwrap_or("Unknown".to_owned()),
                    "artist": player.metadata.artist.unwrap_or(vec!["Unknown".into()]).join(", "),
                    "album": player.metadata.album.unwrap_or("Unknown".to_owned()),
                    "length_ms": player.metadata.length.unwrap_or(0) / 1000,
                    "position_ms": player.position / 1000,
                    "playback_status": player.playback_status.as_string(),
                    "loop": player.loop_status.as_string(),
                    "shuffle": player.shuffle
                }
            })
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

        "get_focused_wayland_window" => json!({
            "success": true,
            "class": "example_app",
            "title": "Example Application",
            "workspace": 1
        }),

        "get_current_datetime" => {
            let now = chrono::Local::now();
            json!({
                "success": true,
                "datetime": now.to_rfc3339()
            })
        },

        _ => json!({
            "success": false,
            "error": format!("Unknown function: {}", name)
        }),
    }
}