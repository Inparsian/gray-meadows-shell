use serde_json::json;
use async_openai::error::OpenAIError;
use async_openai::types::chat::{ChatCompletionTools, ChatCompletionTool, FunctionObjectArgs};

pub fn get_tools() -> Result<Vec<ChatCompletionTools>, OpenAIError> {
    let tools = vec![
        ChatCompletionTools::Function(ChatCompletionTool {
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
        }),
        ChatCompletionTools::Function(ChatCompletionTool {
            function: FunctionObjectArgs::default()
                .name("control_mpris_player")
                .description("Performs an action on the default MPRIS player such as play, pause, toggle play/pause, or skip tracks.")
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
        ChatCompletionTools::Function(ChatCompletionTool {
            function: FunctionObjectArgs::default()
                .name("get_focused_wayland_window")
                .description("Gets the currently focused Wayland window along with the current workspace.")
                .build()?,
        }),
        ChatCompletionTools::Function(ChatCompletionTool {
            function: FunctionObjectArgs::default()
                .name("get_current_datetime")
                .description("Get the current date and time")
                .build()?,
        }),
    ];

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
                    }), |action| match action {
                        "lock" |
                        "logout" |
                        "suspend" |
                        "hibernate" |
                        "reboot" |
                        "shutdown" => success,

                        _ => json!({
                            "success": false,
                            "error": format!("Invalid action: {}", action)
                        }),
                    })
                }
                
                Err(e) => json!({
                    "success": false,
                    "error": format!("Failed to parse arguments: {}", e)
                }),
            }
        },
        "control_mpris_player" |
        "set_mpris_loop_state" |
        "set_mpris_shuffle_state" => success,
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