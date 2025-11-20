//! Configuration file support for anime-watcher.
//!
//! This module provides functionality for loading and saving user preferences
//! from a TOML configuration file.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

/// User configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Translation mode: "sub" or "dub"
    #[serde(default = "default_mode")]
    pub mode: String,

    /// Preferred video quality: "best", "worst", or a number
    #[serde(default = "default_quality")]
    pub quality: String,

    /// Directory for downloads
    #[serde(default = "default_download_dir")]
    pub download_dir: String,

    /// Video player command (overrides platform default)
    #[serde(default)]
    pub player: Option<String>,

    /// Additional arguments to pass to the video player
    #[serde(default)]
    pub player_args: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

fn default_mode() -> String {
    "sub".to_string()
}

fn default_quality() -> String {
    "best".to_string()
}

fn default_download_dir() -> String {
    ".".to_string()
}

impl Config {
    /// Create a new config with default values.
    pub fn new() -> Self {
        Self {
            mode: default_mode(),
            quality: default_quality(),
            download_dir: default_download_dir(),
            player: None,
            player_args: Vec::new(),
        }
    }

    /// Get the path to the config file.
    ///
    /// Returns ~/.config/anime-watcher/config.toml on Linux,
    /// or a platform-appropriate location on other systems.
    pub fn get_config_path() -> Result<PathBuf, io::Error> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "Could not find config directory")
            })?
            .join("anime-watcher");

        Ok(config_dir.join("config.toml"))
    }

    /// Load config from disk.
    ///
    /// Returns default config if the file doesn't exist.
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::get_config_path()?;

        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save config to disk.
    ///
    /// Creates the config directory if it doesn't exist.
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_config_path()?;

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Create a default config file if one doesn't exist.
    ///
    /// Returns the path to the config file.
    pub fn create_default_if_missing() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let path = Self::get_config_path()?;

        if !path.exists() {
            let config = Self::new();
            config.save()?;
        }

        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_config_has_defaults() {
        let config = Config::new();
        assert_eq!(config.mode, "sub");
        assert_eq!(config.quality, "best");
        assert_eq!(config.download_dir, ".");
        assert!(config.player.is_none());
        assert!(config.player_args.is_empty());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            mode: "dub".to_string(),
            quality: "720".to_string(),
            download_dir: "/tmp".to_string(),
            player: Some("vlc".to_string()),
            player_args: vec!["--fullscreen".to_string()],
        };

        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("mode = \"dub\""));
        assert!(toml_str.contains("quality = \"720\""));
        assert!(toml_str.contains("download_dir = \"/tmp\""));
        assert!(toml_str.contains("player = \"vlc\""));
        assert!(toml_str.contains("player_args"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            mode = "dub"
            quality = "1080"
            download_dir = "/downloads"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.mode, "dub");
        assert_eq!(config.quality, "1080");
        assert_eq!(config.download_dir, "/downloads");
        assert!(config.player.is_none());
    }

    #[test]
    fn test_config_partial_deserialization() {
        // Only specify some fields, rest should use defaults
        let toml_str = r#"
            mode = "dub"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.mode, "dub");
        assert_eq!(config.quality, "best"); // default
        assert_eq!(config.download_dir, "."); // default
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.mode, "sub");
        assert_eq!(config.quality, "best");
    }
}
