use serde::Deserialize;
use std::fs;

/// Everything tidewm can be configured with.
/// Stored in %APPDATA%\tidewm\config.toml — falls back to defaults if absent.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Pixel gap between tiles and screen edge
    #[serde(default = "default_gap")]
    pub gap: i32,

    /// Animation duration in milliseconds (0 = instant, no animation)
    #[serde(default = "default_anim_ms")]
    pub animation_ms: u64,

    /// Layout algorithm: "bsp", "tall", "wide", "monocle"
    #[serde(default = "default_layout")]
    pub layout: String,

    /// Main pane ratio (0.0–1.0) for "tall" and "wide" layouts
    #[serde(default = "default_main_ratio")]
    pub main_ratio: f32,

    /// Hotkey modifier: "alt", "win", "ctrl"
    #[serde(default = "default_modifier")]
    pub modifier: String,
}

fn default_gap() -> i32 { 0 }
fn default_anim_ms() -> u64 { 160 }
fn default_layout() -> String { "tall".to_string() }
fn default_main_ratio() -> f32 { 0.55 }
fn default_modifier() -> String { "alt".to_string() }

impl Default for Config {
    fn default() -> Self {
        Self {
            gap: default_gap(),
            animation_ms: default_anim_ms(),
            layout: default_layout(),
            main_ratio: default_main_ratio(),
            modifier: default_modifier(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        if let Ok(text) = fs::read_to_string(&path) {
            match toml::from_str::<Config>(&text) {
                Ok(c) => return c,
                Err(e) => eprintln!("[tidewm] config parse error: {e}"),
            }
        } else {
            // Write default config so the user can find and edit it
            let _ = fs::create_dir_all(path.parent().unwrap());
            let _ = fs::write(&path, DEFAULT_CONFIG);
        }
        Config::default()
    }
}

fn config_path() -> std::path::PathBuf {
    let mut p = dirs_next();
    p.push("tidewm");
    p.push("config.toml");
    p
}

fn dirs_next() -> std::path::PathBuf {
    // %APPDATA%
    std::env::var("APPDATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
}

const DEFAULT_CONFIG: &str = r#"# tidewm configuration
# Edit and save — changes apply on next tidewm start.

# Gap in pixels between tiles and screen edges
gap = 0

# Animation duration in milliseconds (0 = instant)
animation_ms = 160

# Layout: "tall" | "wide" | "bsp" | "monocle"
layout = "tall"

# Main pane ratio for tall/wide layouts (0.0 to 1.0)
main_ratio = 0.55

# Hotkey modifier key: "alt" | "win" | "ctrl"
modifier = "alt"
"#;
