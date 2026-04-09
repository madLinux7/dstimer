# Dead Simple CLI Timer (dstimer)

![Rust](https://img.shields.io/badge/built_with-Rust-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey)

A dead-simple, cross-platform CLI countdown timer featuring:

- Color-changing progress bar
- Fullscreen / Inline mode
- Optional audio playback
- Optional HTTP webhook
- Optional notifications on finish
- YAML-based presets and defaults.

Written in Rust for maximum efficiency and portability.

<p align="center">
  <img src="demo_args_1.gif" alt="dssh demo"><br>
  <sub>Demo with 7 seconds as argument (Fullscreen)</sub>
</p>

<p align="center">
  <img src="demo_pomodoro_yaml.gif" alt="dssh demo"><br>
  <sub>Demo with pomodoro preset (Inline)</sub>
</p>

## Table of Contents

- [Features](#features)
- [Usage](#usage)
  - [TUI / Interactive mode](#tui--interactive-mode-no-arguments)
  - [Fullscreen mode](#fullscreen-mode)
  - [Inline mode](#inline-mode---inline---i)
- [Configuration](#configuration)
  - [defaults.yaml](#defaultsyaml--global-defaults)
  - [presets.yaml](#presetsyaml--named-presets)
- [Install](#install)
- [Supported Audio Formats](#supported-audio-formats)
- [Build from Source](#build-from-source)
- [Contributing](#contributing)
- [Acknowledgements](#-acknowledgements-)

## Features

- Automatically parses **HH:MM:SS**, **MM:SS** or just **seconds**
- **Fullscreen** (default) or **--inline** mode
- Full-width **progress bar**: green → yellow → red as time runs out
- Interactive **time entry** (HH:MM:SS) when no arguments parsed
- ♪ Optional **audio file playback** when the timer completes ♪
- Optional **HTTP call** (`--url`) — fires a GET request on finish and shows the response
- **YAML presets** — save named presets (e.g. `dstimer pomodoro`) in `~/.dstimer/presets.yaml`
- **Global defaults** — set default audio, URL, inline/silent in `~/.dstimer/defaults.yaml`

## Usage

| Flag | Short | Description |
|------|-------|-------------|
| `--time` | `-t` | Duration in `HH:MM:SS`, `MM:SS`, or `SS` format |
| `--audio` | `-a` | Path to audio file to play on finish |
| `--url` | `-u` | URL to call (HTTP GET) when timer finishes |
| `--inline` | `-i` | Inline mode (see below) |
| `--silent` | | Suppress desktop notifications |

The positional argument can be a **time value** (`dstimer 25:00`) or a **preset name** (`dstimer pomodoro`). See [Configuration](#configuration) below.

### **TUI / Interactive mode** (no arguments):

```bash
dstimer
```

You'll be prompted to enter a duration, an optional audio file path, and an optional URL.

![demo_manual](demo_manual.gif)

### Fullscreen mode

```bash
dstimer # starts interactive mode
dstimer 25:00 # 25 minutes
dstimer 7 # 7 seconds
dstimer --time 1:30:17 # 1 hour 30 minutes 17 seconds
dstimer 90 --audio /path/to/audio.wav # plays audio.wav after 90 seconds
dstimer 5:00 --url https://example.com/hook # fires HTTP GET after 5 minutes
dstimer pomodoro # loads named preset
dstimer pomodoro -t 30:00 # preset with CLI time override
```

![Demo with seconds and audio as arguments](demo_args_2.gif)

### **Inline mode** (`--inline` / `-i`):

```bash
dstimer --inline # interactive prompt stays inline too
dstimer 60 -i
```

Renders the timer on the **current terminal line** instead of taking over the full screen. Useful for scripts, split panes, or when you want the rest of your terminal history visible.

![Demo inline interactive mode](demo_inline_manual.gif)
![Demo inline with -i -t 00:00:07 args](demo_inline_args_1.gif)
![Demo inline with -i -t 00:00:07 -a "home/linuxg/Musik/Super Survivor.flac" args](demo_inline_args_2.gif)

## Configuration

dstimer automatically creates `~/.dstimer/` with two YAML files on first run.

### `defaults.yaml` — Global defaults

These values apply to every run unless overridden by a preset or CLI flags.

```yaml
inline: false
silent: false
audio: ""
url: ""
```

When `audio` or `url` are set here, the interactive TUI **skips those prompts** automatically.

### `presets.yaml` — Named presets

Define reusable timer presets and call them by name:

```yaml
pomodoro:
  time: "25:00"
  inline: true
  silent: false
  audio: "/home/user/music/bell.flac"
  url: "https://example.com/pomodoro-done"

break:
  time: "5:00"
  silent: true
```

Then just run:

```bash
dstimer pomodoro           # uses all preset values, starts immediately
dstimer break              # 5-minute silent break timer
dstimer pomodoro -t 30:00  # override just the time
```

All fields in a preset entry are optional. Missing fields fall back to `defaults.yaml`.

**Priority order:** CLI flags > preset > defaults.yaml

## Install

### Linux / macOS Install Script

```sh
curl -fsSL https://raw.githubusercontent.com/madLinux7/dstimer/main/install.sh | sh
```

### macOS

**homebrew**

```sh
brew install madLinux7/tap/dstimer 
```

### Windows

**Winget:**

```ps1
winget install madLinux.dstimer
```

**PowerShell Install Script:**

```ps1
irm https://raw.githubusercontent.com/madLinux7/dstimer/main/install.ps1 | iex
```

### Via Cargo (requires Rust)

```sh
cargo install dstimer
```

## Supported Audio Formats

MP3, FLAC, WAV, OGG, and anything else supported by [Symphonia](https://github.com/pdeljanov/Symphonia).

## Build from Source

```bash
git clone https://github.com/madLinux7/dstimer
cd dstimer
cargo build --release
./target/release/dstimer
```

Requires Rust 1.70+.

## Contributing

Contributions are always welcome! If you want to help, here's the workflow:

1. Fork the repo and create a feature branch
2. `cargo clippy` and `cargo fmt` before opening a PR
3. Follow the existing commit style: `feat:`, `fix:`, `chore:`, `refactor:`

No formal issue template — just open one if you want to discuss an idea first.

## ✨ Acknowledgements ✨

dstimer couldn't be dead simple without the efforts of some great open-source projects:

- [clap](https://github.com/clap-rs/clap) — CLI argument parsing
- [crossterm](https://github.com/crossterm-rs/crossterm) — cross-platform terminal manipulation
- [rodio](https://github.com/RustAudio/rodio) — audio playback
- [Symphonia](https://github.com/pdeljanov/Symphonia) — audio decoding (MP3, FLAC, WAV, OGG, ...)
- [ureq](https://github.com/algesten/ureq) — lightweight HTTP client
- [serde](https://github.com/serde-rs/serde) + [serde_yaml](https://github.com/dtolnay/serde-yaml) — YAML configuration
- [ctrlc](https://github.com/Detegr/rust-ctrlc) — Ctrl+C signal handling
- [notify-rust](https://github.com/hoodie/notify-rust) — desktop notifications on Linux & Windows
- [winresource](https://github.com/mxre/winresource) — embedding the app icon on Windows

And a special shoutout to [VHS](https://github.com/charmbracelet/vhs) by Charm for making it _dead simple_ to record sick custom terminal GIFs!