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
    - [ ] System tray (dbus) (being rewritten to not depend on system-tray crate)
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
    - [ ] System tray ⚠️ 1
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
None.