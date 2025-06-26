#![allow(dead_code)] // This is currently merely a skeleton. This will be removed during development.

use futures_signals::signal::Mutable;
use hyprland::data::{Client, Workspace};
use once_cell::sync::Lazy;

// Wrapper structs to work with Hyprland data and methods
#[derive(Default)]
pub struct Hyprland {
    pub active_client: Option<Mutable<Client>>,
    pub active_workspace: Option<Mutable<Workspace>>,
}

pub static HYPRLAND: Lazy<Hyprland> = Lazy::new(Hyprland::default);