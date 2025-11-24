# Gray Meadows - Desktop Shell

My personal desktop shell using gtk4 and gtk4-layer-shell, re-written from scratch in Rust and C++ (originally in TypeScript + JSX using AGSv2).

## Current status

Currently not ready for daily use yet, but it is getting closer. Technically you could use it right now if you wanted to, but it is not fully feature-complete and there may be bugs.

The To-Do list has been moved to [Trello](https://trello.com/b/bzhLDyI8/gray-meadows-shell-roadmap).

## Optional features

### Mouse event handling
⚠️ **Nevermind, this causes way worse performance issues than I thought, will continue researching this particular area. For the time being just use keyboard shortcuts.**

Due to Wayland's security model, gray-meadows-shell captures mouse events in a bit of a jank way; Instead of hooking into the compositor directly, it instead relies on receiving mouse events from the compositor via IPC. This means that you need to set up your compositor to send mouse events to gray-meadows-shell.

For Hyprland users, add this to your keybinds to enable mouse event handling:

```ini
bindni = , mouse:272, exec, gray-meadows-shell mouse_left_press
bindni = , mouse:273, exec, gray-meadows-shell mouse_right_press
# Release events
bindnir = , mouse:272, exec, gray-meadows-shell mouse_left_release
bindnir = , mouse:273, exec, gray-meadows-shell mouse_right_release
```