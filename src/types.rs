//! Type definitions for the anime-watcher application.
//!
//! This module contains all the core data structures used throughout the application
//! for representing shows, episodes, and stream sources.

use serde::Deserialize;
use std::collections::HashMap;

/// Raw show data as returned from the AllAnime API.
///
/// This struct is used for deserialization and then converted to [`Show`]
/// with the appropriate episode count for the selected mode (sub/dub).
#[derive(Debug, Deserialize)]
pub struct RawShow {
    /// Unique identifier for the show.
    #[serde(rename = "_id")]
    pub id: String,

    /// Display name of the show.
    pub name: String,

    /// Map of translation type to episode count (e.g., "sub" -> 24, "dub" -> 12).
    #[serde(rename = "availableEpisodes")]
    pub available_episodes: HashMap<String, i64>,
}

/// A processed show with episode count for a specific translation mode.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Show {
    /// Unique identifier for the show.
    #[serde(rename = "_id")]
    pub id: String,

    /// Display name of the show.
    pub name: String,

    /// Number of available episodes for the selected translation mode.
    #[serde(rename = "availableEpisodes")]
    pub available_episodes: i64,
}

impl Show {
    /// Format the show for display in selection menus.
    ///
    /// # Examples
    ///
    /// ```
    /// use anime_watcher::types::Show;
    ///
    /// let show = Show {
    ///     id: "abc123".to_string(),
    ///     name: "My Anime".to_string(),
    ///     available_episodes: 24,
    /// };
    /// assert_eq!(show.to_display(), "My Anime (24 eps)");
    /// ```
    pub fn to_display(&self) -> String {
        format!("{} ({} eps)", self.name, self.available_episodes)
    }
}

/// An episode of a show.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Episode {
    /// Unique identifier for the episode.
    pub id: String,

    /// Episode number.
    pub number: i64,

    /// Optional episode title.
    pub title: Option<String>,
}

impl Episode {
    /// Format the episode for display in selection menus.
    ///
    /// # Examples
    ///
    /// ```
    /// use anime_watcher::types::Episode;
    ///
    /// let ep = Episode {
    ///     id: "ep1".to_string(),
    ///     number: 1,
    ///     title: Some("The Beginning".to_string()),
    /// };
    /// assert_eq!(ep.to_display(), "Ep 1 - The Beginning");
    ///
    /// let ep_no_title = Episode {
    ///     id: "ep2".to_string(),
    ///     number: 2,
    ///     title: None,
    /// };
    /// assert_eq!(ep_no_title.to_display(), "Ep 2");
    /// ```
    pub fn to_display(&self) -> String {
        match &self.title {
            Some(t) => format!("Ep {} - {}", self.number, t),
            None => format!("Ep {}", self.number),
        }
    }
}

/// A streaming source for an episode.
#[derive(Clone, Debug, PartialEq)]
pub struct StreamSource {
    /// Video quality (e.g., 1080, 720, 480). 0 indicates unknown quality.
    pub quality: i32,

    /// URL to the video stream or embed page.
    pub url: String,
}

impl StreamSource {
    /// Format the stream source for display in selection menus.
    ///
    /// # Examples
    ///
    /// ```
    /// use anime_watcher::types::StreamSource;
    ///
    /// let source = StreamSource {
    ///     quality: 1080,
    ///     url: "https://example.com/video.mp4".to_string(),
    /// };
    /// assert_eq!(source.to_display(), "1080p");
    ///
    /// let unknown = StreamSource {
    ///     quality: 0,
    ///     url: "https://example.com/video.mp4".to_string(),
    /// };
    /// assert_eq!(unknown.to_display(), "Unknown quality");
    /// ```
    pub fn to_display(&self) -> String {
        if self.quality == 0 {
            "Unknown quality".to_string()
        } else {
            format!("{}p", self.quality)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_to_display() {
        let show = Show {
            id: "abc123".to_string(),
            name: "Test Anime".to_string(),
            available_episodes: 12,
        };
        assert_eq!(show.to_display(), "Test Anime (12 eps)");
    }

    #[test]
    fn test_show_to_display_zero_episodes() {
        let show = Show {
            id: "xyz".to_string(),
            name: "New Show".to_string(),
            available_episodes: 0,
        };
        assert_eq!(show.to_display(), "New Show (0 eps)");
    }

    #[test]
    fn test_episode_to_display_with_title() {
        let ep = Episode {
            id: "ep1".to_string(),
            number: 1,
            title: Some("Pilot".to_string()),
        };
        assert_eq!(ep.to_display(), "Ep 1 - Pilot");
    }

    #[test]
    fn test_episode_to_display_without_title() {
        let ep = Episode {
            id: "ep5".to_string(),
            number: 5,
            title: None,
        };
        assert_eq!(ep.to_display(), "Ep 5");
    }

    #[test]
    fn test_episode_to_display_empty_title() {
        let ep = Episode {
            id: "ep3".to_string(),
            number: 3,
            title: Some("".to_string()),
        };
        assert_eq!(ep.to_display(), "Ep 3 - ");
    }

    #[test]
    fn test_stream_source_creation() {
        let source = StreamSource {
            quality: 1080,
            url: "https://example.com/video.mp4".to_string(),
        };
        assert_eq!(source.quality, 1080);
        assert_eq!(source.url, "https://example.com/video.mp4");
    }

    #[test]
    fn test_stream_source_to_display_with_quality() {
        let source = StreamSource {
            quality: 1080,
            url: "https://example.com/video.mp4".to_string(),
        };
        assert_eq!(source.to_display(), "1080p");
    }

    #[test]
    fn test_stream_source_to_display_720p() {
        let source = StreamSource {
            quality: 720,
            url: "https://example.com/video.mp4".to_string(),
        };
        assert_eq!(source.to_display(), "720p");
    }

    #[test]
    fn test_stream_source_to_display_unknown() {
        let source = StreamSource {
            quality: 0,
            url: "https://example.com/video.mp4".to_string(),
        };
        assert_eq!(source.to_display(), "Unknown quality");
    }
}
