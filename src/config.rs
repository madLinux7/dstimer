use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, io};

const DIR_NAME: &str = ".dstimer";
const DEFAULTS_FILE: &str = "defaults.yaml";
const PRESETS_FILE: &str = "presets.yaml";

#[derive(Serialize, Deserialize)]
pub struct Defaults {
    #[serde(default)]
    pub inline: bool,
    #[serde(default)]
    pub silent: bool,
    #[cfg(feature = "audio")]
    #[serde(default)]
    pub audio: String,
    #[serde(default)]
    pub url: String,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            inline: false,
            silent: false,
            #[cfg(feature = "audio")]
            audio: String::new(),
            url: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct PresetEntry {
    pub time: Option<String>,
    pub inline: Option<bool>,
    pub silent: Option<bool>,
    #[cfg(feature = "audio")]
    pub audio: Option<String>,
    pub url: Option<String>,
}

pub type Presets = HashMap<String, PresetEntry>;

fn config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join(DIR_NAME)
}

fn ensure_dir() -> io::Result<PathBuf> {
    let dir = config_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

pub fn load_defaults() -> Defaults {
    let dir = match ensure_dir() {
        Ok(d) => d,
        Err(_) => return Defaults::default(),
    };
    let path = dir.join(DEFAULTS_FILE);
    if !path.exists() {
        let defaults = Defaults::default();
        let yaml = serde_yaml::to_string(&defaults).unwrap_or_default();
        let _ = fs::write(&path, yaml);
        return defaults;
    }
    let contents = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Defaults::default(),
    };
    serde_yaml::from_str(&contents).unwrap_or_default()
}

pub fn load_presets() -> Presets {
    let dir = match ensure_dir() {
        Ok(d) => d,
        Err(_) => return Presets::new(),
    };
    let path = dir.join(PRESETS_FILE);
    if !path.exists() {
        #[cfg(feature = "audio")]
        let example = "# Named timer presets\n# Example:\n#\n# pomodoro:\n#   time: \"25:00\"\n#   inline: true\n#   silent: false\n#   audio: \"/path/to/sound.flac\"\n#   url: \"\"\n";
        #[cfg(not(feature = "audio"))]
        let example = "# Named timer presets\n# Example:\n#\n# pomodoro:\n#   time: \"25:00\"\n#   inline: true\n#   silent: false\n#   url: \"\"\n";
        let _ = fs::write(&path, example);
        return Presets::new();
    }
    let contents = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Presets::new(),
    };
    serde_yaml::from_str(&contents).unwrap_or_default()
}

/// Resolved effective settings after merging defaults, config entry, and CLI args.
pub struct Effective {
    pub time: Option<String>,
    pub inline: bool,
    pub silent: bool,
    #[cfg(feature = "audio")]
    pub audio: Option<PathBuf>,
    pub url: Option<String>,
}

#[cfg(feature = "audio")]
fn non_empty_path(s: &str) -> Option<PathBuf> {
    if s.is_empty() {
        None
    } else {
        Some(PathBuf::from(s))
    }
}

fn non_empty(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

/// Merge defaults <- preset entry <- CLI args.
/// CLI `Option` args override only when `Some`. Bool flags override only when `true`.
pub fn merge(
    defaults: &Defaults,
    preset: Option<&PresetEntry>,
    cli_inline: bool,
    cli_silent: bool,
    #[cfg(feature = "audio")] cli_audio: Option<PathBuf>,
    cli_url: Option<String>,
) -> Effective {
    let base_inline = preset.and_then(|c| c.inline).unwrap_or(defaults.inline);
    let base_silent = preset.and_then(|c| c.silent).unwrap_or(defaults.silent);
    #[cfg(feature = "audio")]
    let base_audio = preset
        .and_then(|c| c.audio.as_deref())
        .map_or_else(|| non_empty_path(&defaults.audio), non_empty_path);
    let base_url = preset
        .and_then(|c| c.url.as_deref())
        .map_or_else(|| non_empty(&defaults.url), non_empty);

    let preset_time = preset.and_then(|c| c.time.clone());

    Effective {
        time: preset_time,
        inline: cli_inline || base_inline,
        silent: cli_silent || base_silent,
        #[cfg(feature = "audio")]
        audio: cli_audio.or(base_audio),
        url: cli_url.or(base_url),
    }
}

#[cfg(all(test, not(feature = "audio")))]
mod tests {
    use super::*;

    #[test]
    fn accepts_legacy_audio_values_without_enabling_audio() {
        let defaults: Defaults = serde_yaml::from_str(
            "inline: false\nsilent: false\naudio: /tmp/legacy-sound.wav\nurl: https://example.com/default\n",
        )
        .unwrap();
        let presets: Presets = serde_yaml::from_str(
            "headless:\n  time: \"5:00\"\n  audio: /tmp/legacy-preset.wav\n  url: https://example.com/preset\n",
        )
        .unwrap();

        let effective = merge(&defaults, presets.get("headless"), false, false, None);

        assert_eq!(effective.time.as_deref(), Some("5:00"));
        assert_eq!(effective.url.as_deref(), Some("https://example.com/preset"));
    }
}
