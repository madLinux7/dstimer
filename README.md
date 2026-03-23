# Dead Simple CLI Timer (dstimer)

![Rust](https://img.shields.io/badge/built_with-Rust-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey)

A dead-simple, cross-platform CLI countdown timer with color-changing progress bar and optional audio playback on finish.

Written in Rust for maximum efficiency and portability.

![Demo with 7 as argument representing the seconds](demo_args_1.gif)

## Features

- Automatically parses **HH:MM:SS**, **MM:SS** or just **seconds**
- **Fullscreen** (default) or **--inline** mode
- Full-width **progress bar**: green → yellow → red as time runs out
- Interactive **time entry** (HH:MM:SS) when no arguments parsed
- ♪ Optional **audio file playback** when the timer completes ♪

## Usage

| Flag | Short | Description |
|------|-------|-------------|
| `--time` | `-t` | Default argument parsing duration in `HH:MM:SS`, `MM:SS`, or `SS` format |
| `--audio` | `-a` | Path to audio file to play on finish |
| `--inline` | `-i` | Inline mode (see below) |
| `--silent` | | Suppress desktop notifications |

### **Interactive mode** (no arguments):

```bash
dstimer
```

You'll be prompted to enter a duration and an optional audio file path.

![demo_manual](demo_manual.gif)

### Fullscreen mode

```bash
dstimer # starts interactive mode
dstimer 25:00 # 25 minutes
dstimer 7 # 7 seconds
dstimer --time 1:30:17 # 1 hour 30 minutes 17 seconds
dstimer 90 --audio /path/to/audio.wav # plays audio.wav after 90 seconds
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

## Install

**macOS / Linux:**

```bash
curl -fsSL https://raw.githubusercontent.com/madLinux7/dstimer/main/install.sh | sh
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/madLinux7/dstimer/main/install.ps1 | iex
```

**Via Cargo (requires Rust):**

```bash
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
- [ctrlc](https://github.com/Detegr/rust-ctrlc) — Ctrl+C signal handling
- [notify-rust](https://github.com/hoodie/notify-rust) — desktop notifications on Linux & Windows
- [winresource](https://github.com/mxre/winresource) — embedding the app icon on Windows

And a special shoutout to [VHS](https://github.com/charmbracelet/vhs) by Charm for making it _dead simple_ to record bootyful terminal GIFs straight from a script ♥️

## Support Me

If you like using dstimer in your daily routine, consider buying me a coffee or sending a tip:

[Ko-fi](https://ko-fi.com/madlinux) ·
[Bitcoin](bitcoin:bc1qv45u88hnq4xec2l8yx0qfyx88nsn63wxleln0d)

---

Made with ♥️ by [Linus](https://grolmes.de)