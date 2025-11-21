//! TUI type definitions for screens, focus, and actions.

/// The current screen/view of the application.
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    /// Startup screen - choose continue or new search
    Startup,
    /// Search input screen
    Search,
    /// Browsing search results
    ShowList,
    /// Browsing episodes
    EpisodeList,
    /// Selecting quality
    QualitySelect,
    /// Playback menu (after starting stream)
    Playback,
    /// Batch download options
    BatchSelect,
    /// Loading/waiting for API response
    Loading,
}

/// Focus state for split-panel views.
#[derive(Debug, Clone, PartialEq)]
pub enum Focus {
    Sidebar,
    Main,
}

/// Actions that can be returned from the TUI.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Action {
    /// No action, continue running
    None,
    /// Quit the application
    Quit,
    /// Perform a search with the given query
    Search(String),
    /// Select a show by index
    SelectShow(usize),
    /// Select an episode by index
    SelectEpisode(usize),
    /// Select a quality by index
    SelectQuality(usize),
    /// Start streaming the current selection
    Stream,
    /// Download the current selection
    Download,
    /// Play next episode
    Next,
    /// Play previous episode
    Previous,
    /// Replay current episode
    Replay,
    /// Go back to episode selection
    BackToEpisodes,
    /// Continue from history
    ContinueFromHistory(usize),
    /// Start new search
    NewSearch,
    /// Batch download all
    BatchAll,
    /// Batch download range
    BatchRange(i64, i64),
    /// Single download
    BatchSingle,
}
