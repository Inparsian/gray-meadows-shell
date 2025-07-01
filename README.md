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
        - [x] GPU ⚠️ 2
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
1. Several known issues for the system-tray crate (which is what powers the tray singleton) exist, mainly:
- There exist memory leaks due to dangling tray items that aren't cleaned up properly when they're removed. [This is known.](https://github.com/JakeStanger/system-tray/issues/19)
- Tray items can use a high amount of memory due to large icon pixmaps - A notable example is Vesktop, which only has a 1080x1080 icon. This exacerbates the memory leak issue.
- Tray menus for certain applications can break - VLC is a notable example. [This is known.](https://github.com/JakeStanger/system-tray#dbusmenu-gtk3)
- (NITPICK) Tray item updates aren't coalesced, this can lead to unnecessary redraws and processing due to some applications (e.g. VLC) sending bursts of tray item updates. This can admittedly be done in the app itself, but it'd be nice to have this built-in.
2. Only NVIDIA GPUs are supported. I don't have any AMD or Intel GPUs to test with, so I can't implement support for them. If you have an AMD or Intel GPU and want to help, please open an issue or PR.