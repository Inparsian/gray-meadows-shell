use crate::USERNAME;
use crate::singletons::hyprland::HYPRLAND;
use crate::singletons::mpris;

fn us_to_readable_duration(us: u64) -> String {
    let total_seconds = us / 1_000_000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

pub fn transform_variables(input: &str) -> String {
    let mut owned = input.to_owned();

    let now = chrono::Local::now().format("%A, %B %d, %Y at %I:%M %p %Z").to_string(); 

    owned = owned.replace("{USERNAME}", &USERNAME)
                .replace("{DATETIME}", &now);

    if let Some(player) = mpris::get_default_player() {
        owned = owned.replace("{MPRIS_TRACK_TITLE}", &player.metadata.title.unwrap_or("Unknown".to_owned()))
                    .replace("{MPRIS_ARTIST}", &player.metadata.artist.unwrap_or(vec!["Unknown".into()]).join(", "))
                    .replace("{MPRIS_ALBUM}", &player.metadata.album.unwrap_or("Unknown".to_owned()))
                    .replace("{MPRIS_LENGTH_MS}", &(player.metadata.length.unwrap_or(0) / 1000).to_string())
                    .replace("{MPRIS_LENGTH_READABLE}", &us_to_readable_duration(player.metadata.length.unwrap_or(0) as u64))
                    .replace("{MPRIS_POSITION_MS}", &(player.position / 1000).to_string())
                    .replace("{MPRIS_POSITION_READABLE}", &us_to_readable_duration(player.position as u64))
                    .replace("{MPRIS_PLAYBACK_STATUS}", &player.playback_status.as_string())
                    .replace("{MPRIS_LOOP_STATUS}", &player.loop_status.as_string())
                    .replace("{MPRIS_SHUFFLE}", &player.shuffle.to_string());
    } else {
        owned = owned.replace("{MPRIS_TRACK_TITLE}", "No Player")
                    .replace("{MPRIS_ARTIST}", "No Player")
                    .replace("{MPRIS_ALBUM}", "No Player")
                    .replace("{MPRIS_LENGTH_MS}", "0")
                    .replace("{MPRIS_LENGTH_READABLE}", "00:00")
                    .replace("{MPRIS_POSITION_MS}", "0")
                    .replace("{MPRIS_POSITION_READABLE}", "00:00")
                    .replace("{MPRIS_PLAYBACK_STATUS}", "No Player")
                    .replace("{MPRIS_LOOP_STATUS}", "No Player")
                    .replace("{MPRIS_SHUFFLE}", "false");
    }

    if let Some(workspace) = HYPRLAND.active_workspace.get_cloned() {
        owned = owned.replace("{HYPRLAND_ACTIVE_WORKSPACE_ID}", &workspace.id.to_string())
                    .replace("{HYPRLAND_ACTIVE_WORKSPACE_NAME}", &workspace.name)
                    .replace("{HYPRLAND_ACTIVE_WORKSPACE_MONITOR}", &workspace.monitor)
                    .replace("{HYPRLAND_ACTIVE_WORKSPACE_MONITOR_ID}", &workspace.monitor_id.unwrap_or(0).to_string())
                    .replace("{HYPRLAND_ACTIVE_WORKSPACE_WINDOWS}", &workspace.windows.to_string());
    } else {
        owned = owned.replace("{HYPRLAND_ACTIVE_WORKSPACE_ID}", "0")
                    .replace("{HYPRLAND_ACTIVE_WORKSPACE_NAME}", "N/A")
                    .replace("{HYPRLAND_ACTIVE_WORKSPACE_MONITOR}", "N/A")
                    .replace("{HYPRLAND_ACTIVE_WORKSPACE_MONITOR_ID}", "0")
                    .replace("{HYPRLAND_ACTIVE_WORKSPACE_WINDOWS}", "0");
    }

    if let Some(client) = HYPRLAND.active_client.get_cloned() {
        owned = owned.replace("{HYPRLAND_ACTIVE_CLIENT_CLASS}", &client.class)
                    .replace("{HYPRLAND_ACTIVE_CLIENT_TITLE}", &client.title)
                    .replace("{HYPRLAND_ACTIVE_CLIENT_PID}", &client.pid.to_string())
                    .replace("{HYPRLAND_ACTIVE_CLIENT_MONITOR}", &client.monitor.unwrap_or(0).to_string());
    } else {
        owned = owned.replace("{HYPRLAND_ACTIVE_CLIENT_CLASS}", "N/A")
                    .replace("{HYPRLAND_ACTIVE_CLIENT_TITLE}", "N/A")
                    .replace("{HYPRLAND_ACTIVE_CLIENT_PID}", "0")
                    .replace("{HYPRLAND_ACTIVE_CLIENT_MONITOR}", "0");
    }

    owned
}