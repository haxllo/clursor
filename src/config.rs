use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::color::Color;
use crate::error::{AppError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    Hex,
    Rgb,
    Hsl,
}

impl Format {
    pub fn format_color(&self, color: &Color) -> String {
        match self {
            Format::Hex => color.to_hex(),
            Format::Rgb => color.to_rgb_string(),
            Format::Hsl => color.to_hsl_string(),
        }
    }
}

impl Default for Format {
    fn default() -> Self {
        Self::Hex
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Dark,
    Light,
    System,
}

impl Default for Theme {
    fn default() -> Self {
        Self::Dark
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub default_format: Format,
    pub zoom_level: u32,
    pub history_size: usize,
    pub theme: Theme,
    pub copy_on_click: bool,
    pub show_color_name: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_format: Format::Hex,
            zoom_level: 4,
            history_size: 50,
            theme: Theme::Dark,
            copy_on_click: true,
            show_color_name: true,
        }
    }
}

impl Config {
    pub fn config_dir() -> Result<PathBuf> {
        let dir = directories::ProjectDirs::from("", "", "pick")
            .ok_or_else(|| AppError::Config("Cannot determine config directory".into()))?;
        Ok(dir.config_dir().to_path_buf())
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn load() -> Self {
        let path = match Self::config_path() {
            Ok(p) => p,
            Err(e) => {
                warn!("Cannot determine config path: {e}");
                return Self::default();
            }
        };

        match std::fs::read_to_string(&path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(cfg) => cfg,
                Err(e) => {
                    warn!("Config parse error: {e}, using defaults");
                    Self::default()
                }
            },
            Err(_) => {
                // First run or config doesn't exist — save defaults
                let cfg = Self::default();
                let _ = cfg.save();
                cfg
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir()?;
        std::fs::create_dir_all(&dir)?;
        let path = Self::config_path()?;
        let content = toml::to_string_pretty(self)
            .map_err(|e| AppError::Config(format!("Serialization: {e}")))?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}
