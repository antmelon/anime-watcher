//! Integration tests for anime-watcher.
//!
//! These tests verify the integration between different modules
//! using mock data where appropriate.

use anime_watcher::types::{Episode, Show, StreamSource};
use anime_watcher::config::Config;
use anime_watcher::history::WatchHistory;
use anime_watcher::api::Provider;

/// Test that shows can be created and displayed correctly.
#[test]
fn test_show_display_integration() {
    let show = Show {
        id: "test-123".to_string(),
        name: "Test Anime".to_string(),
        available_episodes: 24,
    };

    assert_eq!(show.to_display(), "Test Anime (24 eps)");
    assert_eq!(show.id, "test-123");
}

/// Test episode display formatting.
#[test]
fn test_episode_display_integration() {
    let episode = Episode {
        id: "test-123-1".to_string(),
        number: 1,
        title: Some("Pilot Episode".to_string()),
    };

    assert!(episode.to_display().contains("Ep 1"));
    assert!(episode.to_display().contains("Pilot Episode"));
}

/// Test stream source quality display.
#[test]
fn test_stream_source_quality_integration() {
    let sources = vec![
        StreamSource { quality: 1080, url: "http://example.com/1080p".to_string() },
        StreamSource { quality: 720, url: "http://example.com/720p".to_string() },
        StreamSource { quality: 0, url: "http://example.com/unknown".to_string() },
    ];

    assert_eq!(sources[0].to_display(), "1080p");
    assert_eq!(sources[1].to_display(), "720p");
    assert_eq!(sources[2].to_display(), "Unknown quality");
}

/// Test provider enum parsing and priority.
#[test]
fn test_provider_parsing() {
    assert_eq!(Provider::from_name("Mp4"), Provider::Mp4);
    assert_eq!(Provider::from_name("Fm-Hls"), Provider::FmHls);
    assert!(matches!(Provider::from_name("Unknown"), Provider::Unknown(_)));
}

/// Test provider priority ordering.
#[test]
fn test_provider_priority_ordering() {
    let mp4 = Provider::Mp4;
    let hls = Provider::FmHls;
    let unknown = Provider::from_name("random");

    assert!(mp4.priority() < hls.priority());
    assert!(hls.priority() < unknown.priority());
}

/// Test that provider sorting works correctly.
#[test]
fn test_provider_sorting() {
    let mut providers = vec![
        Provider::from_name("Unknown"),
        Provider::from_name("Fm-Hls"),
        Provider::from_name("Mp4"),
        Provider::from_name("Default"),
    ];

    providers.sort_by_key(|p| p.priority());

    assert_eq!(providers[0], Provider::Mp4);
    assert_eq!(providers[1], Provider::FmHls);
    assert_eq!(providers[2], Provider::Default);
}

/// Test config defaults.
#[test]
fn test_config_defaults() {
    let config = Config::new();

    assert_eq!(config.mode, "sub");
    assert_eq!(config.quality, "best");
    assert_eq!(config.download_dir, ".");
}

/// Test watch history operations.
#[test]
fn test_watch_history_operations() {
    let mut history = WatchHistory::new();

    assert!(history.get_recent(10).is_empty());

    history.update("show-1", "Test Show", 5, "sub");

    let recent = history.get_recent(10);
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].show_id, "show-1");
    assert_eq!(recent[0].episode, 5);
}

/// Test that history correctly sorts by timestamp.
/// This test uses sleeps to ensure different timestamps, so it's ignored by default.
#[test]
#[ignore]
fn test_watch_history_sorting() {
    let mut history = WatchHistory::new();

    // Add shows with delays long enough to guarantee different timestamps (in seconds)
    history.update("show-1", "First", 1, "sub");
    std::thread::sleep(std::time::Duration::from_secs(1));
    history.update("show-2", "Second", 1, "sub");
    std::thread::sleep(std::time::Duration::from_secs(1));
    history.update("show-3", "Third", 1, "sub");

    let recent = history.get_recent(10);

    // Most recent should be first
    assert_eq!(recent[0].show_id, "show-3");
    assert_eq!(recent[1].show_id, "show-2");
    assert_eq!(recent[2].show_id, "show-1");
}

/// Test episode filtering logic.
#[test]
fn test_episode_number_matching() {
    let episodes = vec![
        Episode { id: "1".to_string(), number: 1, title: None },
        Episode { id: "2".to_string(), number: 2, title: None },
        Episode { id: "10".to_string(), number: 10, title: None },
        Episode { id: "11".to_string(), number: 11, title: None },
        Episode { id: "12".to_string(), number: 12, title: None },
    ];

    // Simulate filtering by "1"
    let filter = "1";
    let filtered: Vec<_> = episodes
        .iter()
        .filter(|e| e.number.to_string().contains(filter))
        .collect();

    // Should match 1, 10, 11, 12
    assert_eq!(filtered.len(), 4);
}

/// Test quality selection logic (best quality).
#[test]
fn test_quality_selection_best() {
    let sources = vec![
        StreamSource { quality: 480, url: "480p".to_string() },
        StreamSource { quality: 1080, url: "1080p".to_string() },
        StreamSource { quality: 720, url: "720p".to_string() },
    ];

    let mut known: Vec<_> = sources.iter().filter(|s| s.quality > 0).collect();
    known.sort_by(|a, b| b.quality.cmp(&a.quality));

    // Best should be 1080p
    assert_eq!(known[0].quality, 1080);
}

/// Test quality selection logic (worst quality).
#[test]
fn test_quality_selection_worst() {
    let sources = vec![
        StreamSource { quality: 480, url: "480p".to_string() },
        StreamSource { quality: 1080, url: "1080p".to_string() },
        StreamSource { quality: 720, url: "720p".to_string() },
    ];

    let mut known: Vec<_> = sources.iter().filter(|s| s.quality > 0).collect();
    known.sort_by(|a, b| b.quality.cmp(&a.quality));

    // Worst should be 480p
    assert_eq!(known.last().unwrap().quality, 480);
}

/// Test specific quality selection.
#[test]
fn test_quality_selection_specific() {
    let sources = vec![
        StreamSource { quality: 480, url: "480p".to_string() },
        StreamSource { quality: 1080, url: "1080p".to_string() },
        StreamSource { quality: 720, url: "720p".to_string() },
    ];

    let target = 720;
    let found = sources.iter().find(|s| s.quality == target);

    assert!(found.is_some());
    assert_eq!(found.unwrap().quality, 720);
}

/// Test that URL decoding handles various cases.
#[test]
fn test_url_decoding_integration() {
    // This tests the public interface of the decode function
    use anime_watcher::api::decode_allanime_url;

    // Empty string
    assert_eq!(decode_allanime_url(""), "");

    // Basic hex
    let result = decode_allanime_url("48656c6c6f");
    assert_eq!(result, "Hello");
}
