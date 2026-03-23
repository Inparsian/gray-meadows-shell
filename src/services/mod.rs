pub mod date_time;
pub mod mpris;
pub mod sysstats;
pub mod hyprland;
pub mod tray;
pub mod wireplumber;
pub mod apps;
pub mod calculator;
pub mod g_translate;
pub mod notifications;
pub mod networkmanager;
pub mod clipboard;
pub mod ai;
pub mod weather;
pub mod screen_recorder;

pub async fn activate_all() {
    date_time::activate();
    mpris::activate();
    sysstats::activate();
    hyprland::activate();
    tray::activate();
    wireplumber::activate();
    apps::activate().await;
    calculator::activate();
    g_translate::activate();
    notifications::activate();
    //networkmanager::activate();
    clipboard::activate();
    ai::activate().await;
    weather::activate();
    screen_recorder::activate();
}