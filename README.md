<div align="center">

<img src="assets/waycord.png" alt="WayCord Logo" width="160" height="160" />

# WayCord

### Lightweight, Transparent Discord Voice Overlay for Linux

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Linux-success.svg)](#installation)
[![Wayland](https://img.shields.io/badge/Wayland-Native-informational.svg)](#installation)

*A clean, GPU-accelerated, Discord overlay crafted with pure Rust and egui.*

---

</div>

## Table of Contents
- [Why WayCord?](#-why-waycord)
- [Features](#-features)
- [Discord Client Setup](#-discord-client-setup)
- [Installation](#-installation)
- [Controls & Keybindings](#-controls--keybindings)
- [Configuration](#-configuration)
- [License](#-license)

---

## Why WayCord?

WayCord runs entirely as an **independent, zero-injection desktop overlay**:
- **No Game Hooking:** WayCord runs in its own isolated process without needing `LD_PRELOAD` or graphics pipeline hooks. Avoids anti-cheat false positives.
- **Native Wayland & X11:** Built natively for modern Linux desktops, seamlessly adapting to your display server.
- **Zero-Overhead:** Pure Rust with GPU acceleration using `glutin` and `egui-glow`.
- **Seamless Click-Through:** Floats smoothly above games and desktop apps, passing all mouse clicks and keystrokes directly into your games without interference.

---

## Features

- **3 Distinct Display Styles**:
  - **Transparent Minimal**: Floating avatars and usernames with customizable background tint.
  - **Bubble**: Ultra-compact avatar circles with real-time speaking pulses.
  - **HUD Card**: Clean, structured stream card displaying active channel details, mute/deafen states, and volume meters.
- **5 Handcrafted Color Themes**: Catppuccin Mocha, Tokyo Night, Nord, Gruvbox Dark, and Rosé Pine.
- **Hold `[ALT]` Interactive Mode**:
  - Hold your keyboard's `ALT` key at any time in-game to interact with the overlay.
  - **Drag anywhere** to reposition, or drag the **corner `⌟`** to resize.
  - **Quick Snap**: Snap to the left or right edge of your active monitor with a single click.
  - Release `ALT` to instantly restore click-through mode and save your layout automatically.
- **Smart Screen Snapping & Boundary Clamping**: Automatically identifies your active display and clamps the overlay inside visible space so it can never be lost off-screen.
- **Native System Tray Integration**: Real-time connection status and quick toggles via StatusNotifierItem/DBus (supports KDE, GNOME, Waybar, etc.).
- **Ultra-Low Resource Footprint**: Consumes under 20MB of RAM and negligible CPU/GPU cycles.

---

## Discord Client Setup

Stock official Discord does not support third-party overlay bridges directly. To connect WayCord to your voice chat, please use either:

### 1. Vesktop (Recommended)
- Install Vesktop from [vesktop.dev](https://vesktop.dev), via Flatpak, or your package manager (`yay -S vesktop-bin`).
- Vesktop is an optimized native Discord client for Linux with full Wayland, screen share, and built-in Vencord support.
- **WayCord automatically discovers and hooks into Vesktop with zero extra configuration!**

### 2. Official Discord + Vencord
- If you prefer the standard Discord client, run the official Vencord installer script:
  ```bash
  sh -c "$(curl -sS https://raw.githubusercontent.com/Vendicated/VencordInstaller/main/install.sh)"
  ```
* Select your installed Discord version and choose **Install**.
* WayCord will automatically hook into Vencord's renderer and connect on startup.

> **Note:** Flatpak Vesktop (`dev.vencord.Vesktop`) and Legcord are also fully supported out of the box.

---

## Installation

### Arch Linux (AUR)

WayCord is available on the AUR:

```bash
# Pre-compiled binary (recommended for fast installs):
yay -S waycord-bin

# Or build from source (VCS package):
yay -S waycord-git
```

### GitHub Releases (Pre-compiled Tarball)

Pre-built binaries with desktop integration files and checksums are automatically generated for every release:
- Download the latest `waycord-v*-x86_64-unknown-linux-gnu.tar.gz` from [GitHub Releases](https://github.com/podiax/waycord/releases).
- Extract and run `./waycord`.

### Build from Source

Ensure you have the Rust toolchain and necessary development headers installed:

```bash
# On Arch / Manjaro:
sudo pacman -S base-devel rust cargo libx11 fontconfig

# On Debian / Ubuntu:
sudo apt install build-essential cargo libx11-dev libfontconfig1-dev
```

Clone and build WayCord:

```bash
git clone https://github.com/podiax/waycord.git
cd waycord

cargo build --release
```

The compiled binary will be located at `target/release/waycord`. To install system-wide:

```bash
sudo install -Dm755 target/release/waycord /usr/local/bin/waycord
install -Dm644 assets/waycord.desktop ~/.local/share/applications/waycord.desktop
install -Dm644 assets/waycord.png ~/.local/share/icons/hicolor/256x256/apps/waycord.png
install -Dm644 assets/waycord.svg ~/.local/share/icons/hicolor/scalable/apps/waycord.svg
```

---

## Controls & Keybindings

| Action | Shortcut / Control |
| --- | --- |
| **Interactive Mode** | Hold **`[ALT]`** |
| **Move Overlay** | Hold `[ALT]` + Click & drag anywhere on the overlay |
| **Resize Overlay** | Hold `[ALT]` + Drag bottom-right corner `⌟` |
| **Reset Size** | Hold `[ALT]` + Click `↺ Reset Size` in HUD header or Settings |
| **Toggle Only Speaking** | Hold `[ALT]` + Click checkbox in HUD or Settings |
| **Open Settings** | Hold `[ALT]` + Click `⚙` icon |
| **Snap Left / Right** | Hold `[ALT]` + Click `◀ Snap Left` or `▶ Snap Right` |
| **Restore Click-Through** | Release **`[ALT]`** (all changes save automatically) |

### Command-Line Options

* `waycord`: Standard mode (click-through overlay with global `[ALT]` interactivity and system tray).
* `waycord --demo`: Starts with simulated voice users (ideal for previewing themes and styles offline).
* `waycord --interactive`: Starts with interactive mouse capture enabled immediately without holding `ALT`.

---

## Configuration

Configuration is automatically saved in `~/.config/waycord/config.json`.

Example configuration:

```json
{
  "x": 32.0,
  "y": 48.0,
  "avatar_size": 36.0,
  "style": "Transparent",
  "bg_tint_alpha": 80,
  "theme": "Mocha",
  "opacity": 0.9,
  "show_names": true,
  "only_speaking": false,
  "card_width": 220.0
}
```

---

## License

WayCord is licensed under the MIT License.

