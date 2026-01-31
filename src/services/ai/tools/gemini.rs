// gemini-rust doesn't allow us to define tools using the OpenAPI spec,
// so they are defined differently here. This is a temporary solution until
// I can think of a better way to handle this edge case.

use gemini_rust::{ContentBuilder, FunctionDeclaration, Tool};
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

use crate::config::read_config;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "The action to perform on the player")]
#[serde(rename_all = "lowercase")]
pub enum ControlMprisPlayerAction {
    Toggle,
    Play,
    Pause,
    Stop,
    Next,
    Previous,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ControlMprisPlayer {
    pub action: ControlMprisPlayerAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Requested loop state for the player. Must be one of 'off', 'playlist', or 'track'. If omitted, cycles to the next state.")]
#[serde(rename_all = "lowercase")]
pub enum SetMprisLoopStateAction {
    Off,
    Playlist,
    Track,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetMprisLoopState {
    #[schemars(skip_serializing_if = "Option::is_none")]
    pub loop_state: Option<SetMprisLoopStateAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetMprisShuffleState {
    #[schemars(description = "If provided, sets shuffle to this value; if omitted, the shuffle state will be toggled.")]
    pub shuffle: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Power action to perform")]
#[serde(rename_all = "lowercase")]
pub enum PowerAction {
    Lock,
    Logout,
    Suspend,
    Hibernate,
    Reboot,
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PerformPowerAction {
    pub action: PowerAction,
}

pub fn add_gemini_tools(mut builder: ContentBuilder) -> ContentBuilder {
    let app_config = read_config();

    if app_config.ai.features.mpris_control {
        let control_mpris_player_declaration = FunctionDeclaration::new(
            "control_mpris_player",
            "Performs an action on the default MPRIS player such as play, pause, stop, toggle play/pause, or skip tracks.",
            None
        )
            .with_parameters::<ControlMprisPlayer>();

        let set_mpris_loop_state_declaration = FunctionDeclaration::new(
            "set_mpris_loop_state",
            "Change or cycle the loop state for the default MPRIS player.",
            None
        )
            .with_parameters::<SetMprisLoopState>();

        let set_mpris_shuffle_state_declaration = FunctionDeclaration::new(
            "set_mpris_shuffle_state",
            "Change or toggle the shuffle state for the default MPRIS player.",
            None
        )
            .with_parameters::<SetMprisShuffleState>();

        builder = builder.with_tool(Tool::new(control_mpris_player_declaration));
        builder = builder.with_tool(Tool::new(set_mpris_loop_state_declaration));
        builder = builder.with_tool(Tool::new(set_mpris_shuffle_state_declaration));
    }

    if app_config.ai.features.power_control {
        let perform_power_action_declaration = FunctionDeclaration::new(
            "perform_power_action",
            "Performs a system power action.",
            None
        )
            .with_parameters::<PerformPowerAction>();

        builder = builder.with_tool(Tool::new(perform_power_action_declaration));
    }

    if app_config.weather.enabled && app_config.ai.features.weather_info {
        let weather_tool_declaration = FunctionDeclaration::new(
            "get_current_weather",
            "Fetches the current weather information from the weather service.",
            None
        );

        builder = builder.with_tool(Tool::new(weather_tool_declaration));
    }

    builder
}