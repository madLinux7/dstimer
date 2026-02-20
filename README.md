# dstimer

![Rust](https://img.shields.io/badge/built_with-Rust-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey)

![Demo with only seconds as argument](demo_args_1.gif)

A dead-simple, cross-platform CLI countdown timer with a color-changing progress bar and optional audio playback on finish.

Written in Rust for maximum efficiency and portability.

## Features

- Centered, full-width **progress bar** that shifts green → yellow → red as time runs out
- Interactive **time entry** (HH:MM:SS) when launched with no arguments
- Optional **audio file playback** when the timer completes
- Ctrl+C cancels at any point — including during audio playback

## Usage

### CLI mode

![Demo with seconds and audio as argument](demo_args_2.gif)

```bash
dstimer --seconds 300
dstimer -s 90 --audio /path/to/audio.wav
```

| Flag | Short | Description |
|------|-------|-------------|
| `--seconds` | `-s` | Duration in seconds |
| `--audio` | `-a` | Path to audio file to play on finish |

### **Interactive mode** (no arguments):

![demo_manual](demo_manual.gif)

```bash
dstimer
```

You'll be prompted to enter a duration and an optional audio file path.

## Install

**macOS / Linux:**

```bash
curl -fsSL https://raw.githubusercontent.com/YOUR_GITHUB_USERNAME/dead-simple-cli-timer/main/install.sh | sh
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/YOUR_GITHUB_USERNAME/dead-simple-cli-timer/main/install.ps1 | iex
```

**Via Cargo (requires Rust):**

```bash
cargo install dstimer
```

## Supported Audio Formats

MP3, FLAC, WAV, OGG, and anything else supported by [Symphonia](https://github.com/pdeljanov/Symphonia).

## Build from Source

```bash
git clone <repo-url>
cd dead-simple-cli-timer
cargo build --release
./target/release/dstimer
```

Requires Rust 1.70+.

## Contribute

(TODO: Create pull quest blablabla)

## Support me

(TODO: kofi link, Bitcoin, Ethereum, Monero, Litecoin, Dogecoin, XRP)

---

Made with ❤️ by [Linus Grolmes](https://grolmes.de)