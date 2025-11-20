//! User interface components for the anime-watcher application.
//!
//! This module provides legacy UI types. The main TUI is now in the tui module.

use std::io::{self, Write};

/// Actions available in the post-playback menu.
///
/// After an episode finishes playing, the user can choose one of these actions
/// to control playback flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaybackAction {
    /// Play the next episode in sequence.
    Next,
    /// Replay the current episode.
    Replay,
    /// Play the previous episode.
    Previous,
    /// Return to episode selection menu.
    Select,
    /// Exit the application.
    Quit,
}

/// Actions available when an error occurs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorAction {
    /// Retry the failed operation.
    Retry,
    /// Exit the application.
    Quit,
}

/// Actions available at startup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartupAction {
    /// Continue watching from history.
    Continue,
    /// Start a new search.
    NewSearch,
}

/// Batch download selection mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchMode {
    /// Download all episodes.
    All,
    /// Download a range of episodes.
    Range,
    /// Download single episode (cancel batch).
    Single,
}

/// Prompt the user to enter a search query via stdin.
///
/// Displays a prompt and reads user input from the terminal.
///
/// # Returns
///
/// The trimmed search query string, or an error if reading fails.
///
/// # Examples
///
/// ```no_run
/// let query = anime_watcher::ui::prompt_query()?;
/// println!("You searched for: {}", query);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn prompt_query() -> Result<String, Box<dyn std::error::Error>> {
    print!("Enter anime to search: ");
    io::stdout().flush()?;
    let mut user_query = String::new();
    io::stdin().read_line(&mut user_query)?;
    Ok(user_query.trim().to_string())
}

/// Prompt user for episode range input.
///
/// # Returns
///
/// A tuple of (start, end) episode numbers, or None if cancelled.
pub fn prompt_episode_range() -> Result<(i64, i64), Box<dyn std::error::Error>> {
    print!("Enter episode range (e.g., 1-12): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    let parts: Vec<&str> = input.split('-').collect();
    if parts.len() != 2 {
        return Err("Invalid range format. Use start-end (e.g., 1-12)".into());
    }

    let start: i64 = parts[0].trim().parse()
        .map_err(|_| "Invalid start number")?;
    let end: i64 = parts[1].trim().parse()
        .map_err(|_| "Invalid end number")?;

    if start > end {
        return Err("Start must be less than or equal to end".into());
    }

    Ok((start, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playback_action_equality() {
        assert_eq!(PlaybackAction::Next, PlaybackAction::Next);
        assert_ne!(PlaybackAction::Next, PlaybackAction::Quit);
    }

    #[test]
    fn test_playback_action_clone() {
        let action = PlaybackAction::Replay;
        let cloned = action.clone();
        assert_eq!(action, cloned);
    }

    #[test]
    fn test_playback_action_debug() {
        let action = PlaybackAction::Select;
        let debug_str = format!("{:?}", action);
        assert_eq!(debug_str, "Select");
    }
}
