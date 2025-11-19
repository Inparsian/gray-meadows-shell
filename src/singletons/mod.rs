pub mod date_time;
pub mod mpris;
pub mod sysstats;
pub mod hyprland;
pub mod tray;
pub mod wireplumber;
pub mod apps;
pub mod calculator;
pub mod g_translate;
pub mod networkmanager;

pub fn activate_all() {
    date_time::activate();
    mpris::activate();
    sysstats::activate();
    hyprland::activate();
    tray::activate();
    wireplumber::activate();
    apps::activate();
    calculator::activate();
    g_translate::activate();
    networkmanager::activate();
}