![image](https://github.com/user-attachments/assets/8a0d11cd-6fac-4d7c-832c-12d9f463d28e)

My personal Hyprland desktop shell using gtk4 and gtk4-layer-shell, re-written from scratch in Rust and C++ (originally in TypeScript + JSX using AGSv2).

## Current status

Currently not ready for daily use yet, but it is getting closer. Technically you could use it right now if you wanted to, but it is not fully feature-complete and there may be bugs.

The To-Do list has been moved to [Trello](https://trello.com/b/bzhLDyI8/gray-meadows-shell-roadmap).

Contributions are welcome!

## Why the name "Gray Meadows"?
grayscale color scheme, first screenshot of the agsv2 version used a foggy forest wallpaper. was too lazy to come up with a better name

## Niri support?
Not any time soon.

## Recommended icon theme
Since Gray Meadows is intended to have a monochrome/grayscale aesthetic, I recommend you use an icon theme that matches it. My personal favorite is [Besgnulinux Monochrome](https://www.gnome-look.org/p/2151189/), but any monochrome or grayscale icon theme should work well.

## Building
### Dependencies
There's no exhaustive list of dependencies at the moment, as this project is still in it's infancy. However, I can say with confidence that you will need the following:

- `libqalculate`
- `libadwaita` (you might already have it)
- `cozette-ttf` (primary font)
- `ttf-gohu-nerd` (secondary font for small & big text)
- `gtk4` (of course)
- `gtk4-layer-shell`
- `dart-sass` (for compiling SASS stylesheets)
- `libastal-wireplumber` (for WirePlumber support)

If I've missed any, or any of these are redundant, please open an issue or PR and I'll update this list accordingly.



### Building Gray Meadows
To build Gray Meadows, you will need to have Rust and Cargo installed on your system.
1. Clone the repository:
```bash
git clone https://github.com/inparsian/gray-meadows-shell.git
cd gray-meadows-shell
```
2. Build the project using Cargo:
```bash
cargo build --release
```
3. The compiled binary will be located in the `target/release` directory.

## Running Gray Meadows
To run Gray Meadows, execute the following command from the project root directory:
```bash
# To build the project before running it
cargo run --release

# Or if you just want to execute the binary
./target/release/gray-meadows-shell
```

If you wish to run Gray Meadows when Hyprland starts, you can add it to your Hyprland execs:
```ini
exec-once = /path/to/gray-meadows-shell/target/release/gray-meadows-shell
```
