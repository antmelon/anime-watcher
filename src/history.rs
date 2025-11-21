//! Watch history tracking for anime-watcher.
//!
//! This module provides functionality for saving and loading watch history,
//! allowing users to resume watching from where they left off.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// A record of watching progress for a single show.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchRecord {
    /// Unique identifier for the show.
    pub show_id: String,
    /// Display name of the show.
    pub show_name: String,
    /// Last watched episode number.
    pub episode: i64,
    /// Translation mode used (sub/dub).
    pub mode: String,
    /// Unix timestamp of when this was last watched.
    pub timestamp: u64,
}

/// Watch history containing all watch records.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WatchHistory {
    /// Map of show_id to watch record.
    pub records: HashMap<String, WatchRecord>,
}

impl WatchHistory {
    /// Create a new empty watch history.
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    /// Get the path to the history file.
    ///
    /// Returns ~/.local/share/anime-watcher/history.json on Linux,
    /// or a platform-appropriate location on other systems.
    pub fn get_history_path() -> Result<PathBuf, io::Error> {
        let data_dir = if cfg!(target_os = "linux") {
            dirs::data_local_dir()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "Could not find data directory")
                })?
                .join("anime-watcher")
        } else if cfg!(target_os = "macos") {
            dirs::data_dir()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "Could not find data directory")
                })?
                .join("anime-watcher")
        } else {
            // Windows or other
            dirs::data_local_dir()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "Could not find data directory")
                })?
                .join("anime-watcher")
        };

        Ok(data_dir.join("history.json"))
    }

    /// Load watch history from disk.
    ///
    /// Returns an empty history if the file doesn't exist.
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::get_history_path()?;

        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&path)?;
        let history: WatchHistory = serde_json::from_str(&content)?;
        Ok(history)
    }

    /// Save watch history to disk.
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_history_path()?;

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Update or add a watch record.
    pub fn update(&mut self, show_id: &str, show_name: &str, episode: i64, mode: &str) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let record = WatchRecord {
            show_id: show_id.to_string(),
            show_name: show_name.to_string(),
            episode,
            mode: mode.to_string(),
            timestamp,
        };

        self.records.insert(show_id.to_string(), record);
    }

    /// Get the most recently watched shows, sorted by timestamp.
    pub fn get_recent(&self, limit: usize) -> Vec<&WatchRecord> {
        let mut records: Vec<&WatchRecord> = self.records.values().collect();
        records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        records.truncate(limit);
        records
    }

    /// Get the watch record for a specific show (reserved for future use).
    #[allow(dead_code)]
    pub fn get_record(&self, show_id: &str) -> Option<&WatchRecord> {
        self.records.get(show_id)
    }

    /// Check if there's any watch history (reserved for future use).
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_history_is_empty() {
        let history = WatchHistory::new();
        assert!(history.is_empty());
    }

    #[test]
    fn test_update_adds_record() {
        let mut history = WatchHistory::new();
        history.update("show1", "Test Show", 5, "sub");

        assert!(!history.is_empty());
        let record = history.get_record("show1").unwrap();
        assert_eq!(record.show_name, "Test Show");
        assert_eq!(record.episode, 5);
        assert_eq!(record.mode, "sub");
    }

    #[test]
    fn test_update_overwrites_existing() {
        let mut history = WatchHistory::new();
        history.update("show1", "Test Show", 5, "sub");
        history.update("show1", "Test Show", 10, "sub");

        let record = history.get_record("show1").unwrap();
        assert_eq!(record.episode, 10);
    }

    #[test]
    fn test_get_recent_returns_sorted() {
        let mut history = WatchHistory::new();

        // Manually create records with specific timestamps
        history.records.insert(
            "show1".to_string(),
            WatchRecord {
                show_id: "show1".to_string(),
                show_name: "Show 1".to_string(),
                episode: 1,
                mode: "sub".to_string(),
                timestamp: 1000,
            },
        );
        history.records.insert(
            "show2".to_string(),
            WatchRecord {
                show_id: "show2".to_string(),
                show_name: "Show 2".to_string(),
                episode: 2,
                mode: "sub".to_string(),
                timestamp: 2000,
            },
        );
        history.records.insert(
            "show3".to_string(),
            WatchRecord {
                show_id: "show3".to_string(),
                show_name: "Show 3".to_string(),
                episode: 3,
                mode: "sub".to_string(),
                timestamp: 3000,
            },
        );

        let recent = history.get_recent(2);
        assert_eq!(recent.len(), 2);
        // Most recent (highest timestamp) should be first
        assert_eq!(recent[0].show_id, "show3");
        assert_eq!(recent[1].show_id, "show2");
    }

    #[test]
    fn test_get_record_not_found() {
        let history = WatchHistory::new();
        assert!(history.get_record("nonexistent").is_none());
    }
}
