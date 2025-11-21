//! Configuration file support for anime-watcher.
//!
//! This module provides functionality for loading and saving user preferences
//! from a TOML configuration file.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

/// A key binding that can match against key events.
/// Supports format like "j", "Enter", "Esc", "Ctrl+c", "Up", "Down", etc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct KeyBinding(pub String);

impl KeyBinding {
    /// Check if this binding matches the given key event.
    ///
    /// Matches the key code and modifiers. SHIFT is allowed to pass through
    /// since it affects character case. ALT and META must not be present
    /// unless explicitly specified in the binding.
    ///
    /// # Examples
    ///
    /// ```
    /// use anime_watcher::config::KeyBinding;
    /// use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    ///
    /// let binding = KeyBinding("j".to_string());
    /// let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
    /// assert!(binding.matches(&key));
    /// ```
    pub fn matches(&self, key: &KeyEvent) -> bool {
        let binding = self.0.to_lowercase();

        // Check for modifier prefixes
        let (has_ctrl, key_part) = if binding.starts_with("ctrl+") {
            (true, &binding[5..])
        } else {
            (false, binding.as_str())
        };

        // Verify CONTROL modifier matches the binding intent
        if has_ctrl != key.modifiers.contains(KeyModifiers::CONTROL) {
            return false;
        }

        // Reject unexpected modifiers (ALT, META) when binding doesn't specify them
        // SHIFT is allowed since it affects character case
        if !has_ctrl {
            // When binding has no modifiers, reject ALT and META
            if key.modifiers.contains(KeyModifiers::ALT)
                || key.modifiers.contains(KeyModifiers::META)
            {
                return false;
            }
        } else {
            // When binding has Ctrl, also reject ALT and META
            if key.modifiers.contains(KeyModifiers::ALT)
                || key.modifiers.contains(KeyModifiers::META)
            {
                return false;
            }
        }

        // Match the key code
        match key_part {
            "enter" => key.code == KeyCode::Enter,
            "esc" | "escape" => key.code == KeyCode::Esc,
            "tab" => key.code == KeyCode::Tab,
            "backspace" => key.code == KeyCode::Backspace,
            "up" => key.code == KeyCode::Up,
            "down" => key.code == KeyCode::Down,
            "left" => key.code == KeyCode::Left,
            "right" => key.code == KeyCode::Right,
            "space" => key.code == KeyCode::Char(' '),
            s if s.len() == 1 => {
                if let Some(c) = s.chars().next() {
                    key.code == KeyCode::Char(c)
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

/// Custom keybindings configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybindings {
    // Navigation
    /// Move up in lists
    #[serde(default = "default_up")]
    pub up: Vec<KeyBinding>,
    /// Move down in lists
    #[serde(default = "default_down")]
    pub down: Vec<KeyBinding>,
    /// Select/confirm
    #[serde(default = "default_select")]
    pub select: Vec<KeyBinding>,
    /// Go back
    #[serde(default = "default_back")]
    pub back: Vec<KeyBinding>,
    /// Quit application
    #[serde(default = "default_quit")]
    pub quit: Vec<KeyBinding>,

    // Search
    /// Focus search bar
    #[serde(default = "default_search")]
    pub search: Vec<KeyBinding>,

    // UI
    /// Toggle focus between sidebar and main
    #[serde(default = "default_toggle_focus")]
    pub toggle_focus: Vec<KeyBinding>,
    /// Show help
    #[serde(default = "default_help")]
    pub help: Vec<KeyBinding>,

    // Episode list
    /// Filter episodes
    #[serde(default = "default_filter")]
    pub filter: Vec<KeyBinding>,

    // Playback menu
    /// Next episode
    #[serde(default = "default_next")]
    pub next: Vec<KeyBinding>,
    /// Previous episode
    #[serde(default = "default_previous")]
    pub previous: Vec<KeyBinding>,
    /// Replay episode
    #[serde(default = "default_replay")]
    pub replay: Vec<KeyBinding>,
    /// Back to episode selection
    #[serde(default = "default_episodes")]
    pub episodes: Vec<KeyBinding>,

    // Startup
    /// New search from startup
    #[serde(default = "default_new_search")]
    pub new_search: Vec<KeyBinding>,
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            up: default_up(),
            down: default_down(),
            select: default_select(),
            back: default_back(),
            quit: default_quit(),
            search: default_search(),
            toggle_focus: default_toggle_focus(),
            help: default_help(),
            filter: default_filter(),
            next: default_next(),
            previous: default_previous(),
            replay: default_replay(),
            episodes: default_episodes(),
            new_search: default_new_search(),
        }
    }
}

impl Keybindings {
    /// Check if any binding in the list matches the key event.
    pub fn matches(&self, bindings: &[KeyBinding], key: &KeyEvent) -> bool {
        bindings.iter().any(|b| b.matches(key))
    }
}

// Default keybinding functions

/// Returns the default keybindings for moving up in lists.
fn default_up() -> Vec<KeyBinding> {
    vec![KeyBinding("k".to_string()), KeyBinding("Up".to_string())]
}

/// Returns the default keybindings for moving down in lists.
fn default_down() -> Vec<KeyBinding> {
    vec![KeyBinding("j".to_string()), KeyBinding("Down".to_string())]
}

/// Returns the default keybindings for selecting/confirming.
fn default_select() -> Vec<KeyBinding> {
    vec![KeyBinding("Enter".to_string())]
}

/// Returns the default keybindings for going back.
fn default_back() -> Vec<KeyBinding> {
    vec![
        KeyBinding("Backspace".to_string()),
        KeyBinding("Esc".to_string()),
    ]
}

/// Returns the default keybindings for quitting the application.
fn default_quit() -> Vec<KeyBinding> {
    vec![KeyBinding("q".to_string()), KeyBinding("Esc".to_string())]
}

/// Returns the default keybindings for focusing the search bar.
fn default_search() -> Vec<KeyBinding> {
    vec![KeyBinding("s".to_string()), KeyBinding("/".to_string())]
}

/// Returns the default keybindings for toggling focus between sidebar and main.
fn default_toggle_focus() -> Vec<KeyBinding> {
    vec![KeyBinding("Tab".to_string())]
}

/// Returns the default keybindings for showing help.
fn default_help() -> Vec<KeyBinding> {
    vec![KeyBinding("?".to_string())]
}

/// Returns the default keybindings for filtering episodes.
fn default_filter() -> Vec<KeyBinding> {
    vec![KeyBinding("f".to_string())]
}

/// Returns the default keybindings for playing the next episode.
fn default_next() -> Vec<KeyBinding> {
    vec![KeyBinding("n".to_string())]
}

/// Returns the default keybindings for playing the previous episode.
fn default_previous() -> Vec<KeyBinding> {
    vec![KeyBinding("p".to_string())]
}

/// Returns the default keybindings for replaying the current episode.
fn default_replay() -> Vec<KeyBinding> {
    vec![KeyBinding("r".to_string())]
}

/// Returns the default keybindings for returning to episode selection.
fn default_episodes() -> Vec<KeyBinding> {
    vec![KeyBinding("e".to_string())]
}

/// Returns the default keybindings for starting a new search.
fn default_new_search() -> Vec<KeyBinding> {
    vec![KeyBinding("s".to_string()), KeyBinding("n".to_string())]
}

/// Color scheme configuration for the TUI.
///
/// Colors can be specified as:
/// - Named colors: "Black", "Red", "Green", "Yellow", "Blue", "Magenta", "Cyan", "Gray", "White"
/// - Dark variants: "DarkGray", "LightRed", "LightGreen", "LightYellow", "LightBlue", "LightMagenta", "LightCyan"
/// - RGB hex: "#ff0000" or "#f00"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    /// Border color when focused
    #[serde(default = "default_border_focused")]
    pub border_focused: String,
    /// Border color when unfocused
    #[serde(default = "default_border_unfocused")]
    pub border_unfocused: String,
    /// Title and highlight color
    #[serde(default = "default_highlight")]
    pub highlight: String,
    /// Selected item background
    #[serde(default = "default_selection_bg")]
    pub selection_bg: String,
    /// Normal text color
    #[serde(default = "default_text")]
    pub text: String,
    /// Dimmed/inactive text color
    #[serde(default = "default_text_dim")]
    pub text_dim: String,
    /// Error message color
    #[serde(default = "default_error")]
    pub error: String,
    /// Status message color
    #[serde(default = "default_status")]
    pub status: String,
    /// Mode indicator color
    #[serde(default = "default_mode_color")]
    pub mode_indicator: String,
    /// Streaming mode indicator
    #[serde(default = "default_streaming")]
    pub streaming: String,
    /// Download mode indicator
    #[serde(default = "default_download_color")]
    pub download: String,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            border_focused: default_border_focused(),
            border_unfocused: default_border_unfocused(),
            highlight: default_highlight(),
            selection_bg: default_selection_bg(),
            text: default_text(),
            text_dim: default_text_dim(),
            error: default_error(),
            status: default_status(),
            mode_indicator: default_mode_color(),
            streaming: default_streaming(),
            download: default_download_color(),
        }
    }
}

impl ColorScheme {
    /// Parse a color string into a ratatui Color.
    ///
    /// Supports named colors and hex RGB values.
    pub fn parse_color(s: &str) -> Color {
        match s.to_lowercase().as_str() {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "gray" | "grey" => Color::Gray,
            "darkgray" | "darkgrey" => Color::DarkGray,
            "lightred" => Color::LightRed,
            "lightgreen" => Color::LightGreen,
            "lightyellow" => Color::LightYellow,
            "lightblue" => Color::LightBlue,
            "lightmagenta" => Color::LightMagenta,
            "lightcyan" => Color::LightCyan,
            "white" => Color::White,
            hex if hex.starts_with('#') => {
                if let Ok(rgb) = Self::parse_hex(hex) {
                    Color::Rgb(rgb.0, rgb.1, rgb.2)
                } else {
                    Color::White
                }
            }
            _ => Color::White,
        }
    }

    /// Parse a hex color string to RGB values.
    ///
    /// Accepts strings with or without a leading '#'.
    fn parse_hex(hex: &str) -> Result<(u8, u8, u8), ()> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).map_err(|_| ())? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).map_err(|_| ())? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).map_err(|_| ())? * 17;
                Ok((r, g, b))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ())?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ())?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ())?;
                Ok((r, g, b))
            }
            _ => Err(()),
        }
    }

    /// Get the border_focused color as a ratatui Color.
    pub fn border_focused(&self) -> Color {
        Self::parse_color(&self.border_focused)
    }

    /// Get the border_unfocused color as a ratatui Color.
    pub fn border_unfocused(&self) -> Color {
        Self::parse_color(&self.border_unfocused)
    }

    /// Get the highlight color as a ratatui Color.
    pub fn highlight(&self) -> Color {
        Self::parse_color(&self.highlight)
    }

    /// Get the selection_bg color as a ratatui Color.
    pub fn selection_bg(&self) -> Color {
        Self::parse_color(&self.selection_bg)
    }

    /// Get the text color as a ratatui Color.
    pub fn text(&self) -> Color {
        Self::parse_color(&self.text)
    }

    /// Get the text_dim color as a ratatui Color.
    pub fn text_dim(&self) -> Color {
        Self::parse_color(&self.text_dim)
    }

    /// Get the error color as a ratatui Color.
    pub fn error(&self) -> Color {
        Self::parse_color(&self.error)
    }

    /// Get the status color as a ratatui Color.
    pub fn status(&self) -> Color {
        Self::parse_color(&self.status)
    }

    /// Get the mode_indicator color as a ratatui Color.
    pub fn mode_indicator(&self) -> Color {
        Self::parse_color(&self.mode_indicator)
    }

    /// Get the streaming color as a ratatui Color.
    pub fn streaming(&self) -> Color {
        Self::parse_color(&self.streaming)
    }

    /// Get the download color as a ratatui Color.
    pub fn download(&self) -> Color {
        Self::parse_color(&self.download)
    }
}

// Default color functions

/// Returns the default focused border color.
fn default_border_focused() -> String {
    "Cyan".to_string()
}

/// Returns the default unfocused border color.
fn default_border_unfocused() -> String {
    "DarkGray".to_string()
}

/// Returns the default highlight color.
fn default_highlight() -> String {
    "Yellow".to_string()
}

/// Returns the default selection background color.
fn default_selection_bg() -> String {
    "DarkGray".to_string()
}

/// Returns the default text color.
fn default_text() -> String {
    "White".to_string()
}

/// Returns the default dimmed text color.
fn default_text_dim() -> String {
    "DarkGray".to_string()
}

/// Returns the default error color.
fn default_error() -> String {
    "Red".to_string()
}

/// Returns the default status message color.
fn default_status() -> String {
    "Yellow".to_string()
}

/// Returns the default mode indicator color.
fn default_mode_color() -> String {
    "Magenta".to_string()
}

/// Returns the default streaming indicator color.
fn default_streaming() -> String {
    "Green".to_string()
}

/// Returns the default download indicator color.
fn default_download_color() -> String {
    "Red".to_string()
}

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

    /// Log verbosity level: 0=error, 1=warn, 2=info, 3=debug, 4=trace
    #[serde(default = "default_log_level")]
    pub log_level: u8,

    /// Custom keybindings
    #[serde(default)]
    pub keybindings: Keybindings,

    /// Color scheme for the TUI
    #[serde(default)]
    pub colors: ColorScheme,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the default translation mode (subtitled).
fn default_mode() -> String {
    "sub".to_string()
}

/// Returns the default video quality preference.
fn default_quality() -> String {
    "best".to_string()
}

/// Returns the default download directory (current directory).
fn default_download_dir() -> String {
    ".".to_string()
}

/// Returns the default log level (warn).
fn default_log_level() -> u8 {
    1
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
            log_level: default_log_level(),
            keybindings: Keybindings::default(),
            colors: ColorScheme::default(),
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

    /// Save config to disk (reserved for future use).
    ///
    /// Creates the config directory if it doesn't exist.
    #[allow(dead_code)]
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

    /// Create a default config file if one doesn't exist (reserved for future use).
    ///
    /// Returns the path to the config file.
    #[allow(dead_code)]
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
            log_level: 2,
            keybindings: Keybindings::default(),
            colors: ColorScheme::default(),
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

    #[test]
    fn test_keybinding_matches_char() {
        let binding = KeyBinding("j".to_string());
        let key_j = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let key_k = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);

        assert!(binding.matches(&key_j));
        assert!(!binding.matches(&key_k));
    }

    #[test]
    fn test_keybinding_matches_special_keys() {
        let enter = KeyBinding("Enter".to_string());
        let esc = KeyBinding("Esc".to_string());
        let tab = KeyBinding("Tab".to_string());
        let backspace = KeyBinding("Backspace".to_string());

        assert!(enter.matches(&KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)));
        assert!(esc.matches(&KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));
        assert!(tab.matches(&KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)));
        assert!(backspace.matches(&KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)));
    }

    #[test]
    fn test_keybinding_matches_arrow_keys() {
        let up = KeyBinding("Up".to_string());
        let down = KeyBinding("Down".to_string());

        assert!(up.matches(&KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)));
        assert!(down.matches(&KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)));
        assert!(!up.matches(&KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)));
    }

    #[test]
    fn test_keybinding_matches_ctrl_modifier() {
        let ctrl_c = KeyBinding("Ctrl+c".to_string());

        let key_ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let key_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);

        assert!(ctrl_c.matches(&key_ctrl_c));
        assert!(!ctrl_c.matches(&key_c));
    }

    #[test]
    fn test_keybinding_rejects_alt_meta_modifiers() {
        let binding = KeyBinding("j".to_string());

        // Should match with no modifiers
        assert!(binding.matches(&KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)));

        // Should match with SHIFT (SHIFT affects character case)
        assert!(binding.matches(&KeyEvent::new(KeyCode::Char('j'), KeyModifiers::SHIFT)));

        // Should reject ALT
        assert!(!binding.matches(&KeyEvent::new(KeyCode::Char('j'), KeyModifiers::ALT)));

        // Should reject META
        assert!(!binding.matches(&KeyEvent::new(KeyCode::Char('j'), KeyModifiers::META)));

        // Should reject ALT+SHIFT
        let alt_shift = KeyModifiers::ALT | KeyModifiers::SHIFT;
        assert!(!binding.matches(&KeyEvent::new(KeyCode::Char('j'), alt_shift)));
    }

    #[test]
    fn test_keybinding_ctrl_rejects_alt_meta() {
        let ctrl_c = KeyBinding("Ctrl+c".to_string());

        // Should match Ctrl+c
        assert!(ctrl_c.matches(&KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)));

        // Should reject Ctrl+Alt+c
        let ctrl_alt = KeyModifiers::CONTROL | KeyModifiers::ALT;
        assert!(!ctrl_c.matches(&KeyEvent::new(KeyCode::Char('c'), ctrl_alt)));

        // Should reject Ctrl+Meta+c
        let ctrl_meta = KeyModifiers::CONTROL | KeyModifiers::META;
        assert!(!ctrl_c.matches(&KeyEvent::new(KeyCode::Char('c'), ctrl_meta)));
    }

    #[test]
    fn test_keybinding_case_insensitive() {
        let enter_lower = KeyBinding("enter".to_string());
        let enter_upper = KeyBinding("ENTER".to_string());

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        assert!(enter_lower.matches(&key));
        assert!(enter_upper.matches(&key));
    }

    #[test]
    fn test_keybindings_matches_helper() {
        let keybindings = Keybindings::default();

        let key_j = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let key_down = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let key_x = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);

        assert!(keybindings.matches(&keybindings.down, &key_j));
        assert!(keybindings.matches(&keybindings.down, &key_down));
        assert!(!keybindings.matches(&keybindings.down, &key_x));
    }

    #[test]
    fn test_default_keybindings() {
        let kb = Keybindings::default();

        // Test default values have expected keys
        assert_eq!(kb.up.len(), 2);
        assert_eq!(kb.down.len(), 2);
        assert_eq!(kb.select.len(), 1);
        assert_eq!(kb.quit.len(), 2);
    }

    #[test]
    fn test_keybindings_deserialization() {
        let toml_str = r#"
            [keybindings]
            up = ["w", "Up"]
            down = ["s", "Down"]
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.keybindings.up.len(), 2);
        assert_eq!(config.keybindings.up[0].0, "w");
        assert_eq!(config.keybindings.down[0].0, "s");
        // Other keybindings should use defaults
        assert_eq!(config.keybindings.select.len(), 1);
    }

    #[test]
    fn test_keybindings_partial_override() {
        let toml_str = r#"
            mode = "sub"
            [keybindings]
            quit = ["x"]
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        // quit should be overridden
        assert_eq!(config.keybindings.quit.len(), 1);
        assert_eq!(config.keybindings.quit[0].0, "x");
        // up should still have defaults
        assert_eq!(config.keybindings.up.len(), 2);
    }
}
