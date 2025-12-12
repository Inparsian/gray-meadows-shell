<div align="center">
  <h1><img width="700" alt="image" src="https://github.com/user-attachments/assets/16859424-f9f0-4b0d-8c8a-0f77c98331ec" /></h1>
  <img alt="GitHub License" src="https://img.shields.io/github/license/Inparsian/gray-meadows-shell">
  <img alt="GitHub top language" src="https://img.shields.io/github/languages/top/Inparsian/gray-meadows-shell">
  <img alt="GitHub contributors" src="https://img.shields.io/github/contributors/Inparsian/gray-meadows-shell"/>
  <img alt="GitHub commit activity" src="https://img.shields.io/github/commit-activity/w/Inparsian/gray-meadows-shell"/>
  <img alt="GitHub Repo stars" src="https://img.shields.io/github/stars/Inparsian/gray-meadows-shell?style=flat"/>
  <img alt="GitHub forks" src="https://img.shields.io/github/forks/Inparsian/gray-meadows-shell?style=flat"/>
  <img alt="GitHub issues" src="https://img.shields.io/github/issues/Inparsian/gray-meadows-shell"/>
  <img alt="GitHub pull requests" src="https://img.shields.io/github/issues-pr/Inparsian/gray-meadows-shell"/>
</div>
<br>

My personal stand-alone Hyprland desktop shell using gtk4 and gtk4-layer-shell, re-written from scratch in Rust and C++ (originally in TypeScript + JSX using AGSv2).

## 📷 Show me the screenshots
ok

<table align="center">
  <tr>
    <td colspan="1">
      <img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/77459e91-98fc-4080-ac3f-304816f52d64" />
    </td>
    <td colspan="1">
      <img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/d6bd4dc4-6f7e-45c4-aadb-c61d61622aa5" />
    </td>
  </tr>
  <tr>
    <td colspan="1">
      <img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/87d6df6e-7bb2-41d8-a8c1-e5001a69d261" />
    </td>
    <td colspan="1">
      <img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/fe30025e-d024-4757-ba01-5bf306996672" />
    </td>
  </tr>
</table>

## 🛠️ Current status

Currently not ready for daily use yet, but it is getting closer. Technically you could use it right now if you wanted to, but it is not fully feature-complete and there may be bugs.

The To-Do list has been moved to [Trello](https://trello.com/b/bzhLDyI8/gray-meadows-shell-roadmap).

Contributions are welcome!

## 🤔 Why the name "Gray Meadows"?
grayscale color scheme, first screenshot of the agsv2 version used a foggy forest wallpaper. was too lazy to come up with a better name

## 🤔 Niri support?
Not any time soon.

## 🔵 Recommended icon theme
Since Gray Meadows is intended to have a monochrome/grayscale aesthetic, I recommend you use an icon theme that matches it. My personal favorite is [Besgnulinux Monochrome](https://www.gnome-look.org/p/2151189/), but any monochrome or grayscale icon theme should work well.

## 🔨 Building
### Dependencies
There's no exhaustive list of dependencies at the moment, as this project is still in it's infancy. However, I can say with confidence that you will need the following:

#### Required

- `libqalculate`
- `libadwaita`
- `gtk4` (of course)
- `gtk4-layer-shell`
- `dart-sass` (for compiling SASS stylesheets)
- `libastal-wireplumber` (for WirePlumber support)

#### Optional
- `cozette-ttf` (recommended primary font)
- `ttf-gohu-nerd` (recommended secondary font for small & big text)
- `cliphist` and `wl-clipboard` (for clipboard history support)

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

## ⚙️ Running Gray Meadows
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
