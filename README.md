# Gray Meadows - Desktop Shell

My personal desktop shell using gtk4 and gtk4-layer-shell, re-written from scratch in Rust and C++ (originally in TypeScript + JSX using AGSv2).

## To-Do

Currently far from ready for daily use. This list of things to do may not be exhaustive and is subject to change.

- [ ] IPC (so the compositor can communicate with the shell)
- [ ] Library/singleton services
    - [x] libqalculate (Thin C++ shim)
    - [x] Mpris (dbus)
    - [ ] Cava?
    - [ ] Notifications (dbus)
    - [ ] System tray (dbus)
    - [ ] Hyprland (IPC)
    - [ ] Apps
    - [ ] WirePlumber
    - [ ] NetworkManager
    - [ ] System resource monitoring
    - [ ] Timers
    - [ ] Weather
- [ ] Bar
    - [ ] Workspaces
    - [ ] Active client
    - [ ] Resources
    - [ ] Mpris (partially done)
    - [x] Clock
    - [ ] System tray
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