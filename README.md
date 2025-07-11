# Gray Meadows - Desktop Shell

My personal desktop shell using gtk4 and gtk4-layer-shell, re-written from scratch in Rust and C++ (originally in TypeScript + JSX using AGSv2).

## To-Do

Currently far from ready for daily use. This list of things to do may not be exhaustive and is subject to change.

⚠️ = Known bugs/issues exist.<br>
❓ = This particular module will be a challenge.

- [x] IPC (so the compositor can communicate with the shell)
- [ ] Library/singleton services
    - [x] libqalculate (C++ FFI)
    - [x] Mpris (dbus)
    - [ ] Cava?
    - [ ] Notifications (dbus)
    - [x] System tray (dbus)
    - [x] Hyprland (IPC)
    - [x] Apps
    - [x] WirePlumber (C++ FFI)
    - [ ] NetworkManager (dbus?)
    - [x] System resource monitoring
        - [x] Built-in (CPU, RAM)
        - [x] CPU Temp (and maybe other temp sensors)
        - [x] GPU ⚠️ 1
    - [ ] Timers
    - [ ] Weather
- [ ] Bar
    - [x] Workspaces
    - [x] Active client
    - [x] Resources
    - [ ] Mpris (partially done)
    - [x] Clock
    - [x] System tray
    - [x] Default output volume
    - [ ] Indicators
- [ ] OSD
    - [ ] Mpris
    - [ ] Volume
    - [ ] Keybinds
    - [ ] Notifications
- [ ] Overview
    - [ ] Search
- [x] Power menu
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
1. Only NVIDIA GPUs are supported. I don't have any AMD or Intel GPUs to test with, so I can't implement support for them. If you have an AMD or Intel GPU and want to help, please open an issue or PR.

### ❓ Known challenges
T.B.A.