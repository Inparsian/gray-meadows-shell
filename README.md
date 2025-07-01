# Gray Meadows - Desktop Shell

My personal desktop shell using gtk4 and gtk4-layer-shell, re-written from scratch in Rust and C++ (originally in TypeScript + JSX using AGSv2).

## To-Do

Currently far from ready for daily use. This list of things to do may not be exhaustive and is subject to change.

⚠️ = Known bugs exist.

- [ ] IPC (so the compositor can communicate with the shell)
- [ ] Library/singleton services
    - [x] libqalculate (Thin C++ shim)
    - [x] Mpris (dbus)
    - [ ] Cava?
    - [ ] Notifications (dbus)
    - [x] System tray (dbus) ⚠️ 1
    - [x] Hyprland (IPC)
    - [ ] Apps
    - [ ] WirePlumber
    - [ ] NetworkManager
    - [x] System resource monitoring
        - [x] Built-in (CPU, RAM)
        - [x] CPU Temp (and maybe other temp sensors)
        - [x] GPU
    - [ ] Timers
    - [ ] Weather
- [ ] Bar
    - [x] Workspaces
    - [x] Active client
    - [x] Resources
    - [ ] Mpris (partially done)
    - [x] Clock
    - [x] System tray ⚠️ 1
    - [ ] Default output volume
    - [ ] Indicators
- [ ] OSD
    - [ ] Mpris
    - [ ] Volume
    - [ ] Keybinds
    - [ ] Notifications
- [ ] Overview
    - [ ] Search
- [ ] Power menu
- [ ] Right sidebar
    - [ ] Header
    - [ ] Quick toggles
    - [ ] Top modules
        - [ ] Notifications
        - [ ] Volume mixer
        - [ ] Wifi
    - [ ] Bottom modules
        - [ ] Calendar
        - [ ] Weather
        - [ ] Timers
- [ ] Left sidebar
    - [ ] Google Translate
    - [ ] Color picker

### ⚠️ Known issues
1. Tray menus for certain applications can break - VLC is a notable example. This is a known issue in the system-tray crate, which is what powers the tray singleton.