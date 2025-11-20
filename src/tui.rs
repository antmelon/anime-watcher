//! Terminal User Interface for anime-watcher using ratatui.
//!
//! This module provides a full-screen TUI with panels for browsing
//! and selecting anime shows and episodes.

use crate::types::{Episode, Show, StreamSource};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use std::io;
use std::time::Duration;

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

/// Application state for the TUI.
pub struct App {
    /// Current screen being displayed
    pub screen: Screen,
    /// Current focus (sidebar or main)
    pub focus: Focus,
    /// Whether the app should quit
    pub should_quit: bool,
    /// Current search query being typed
    pub search_input: String,
    /// Whether search bar is focused
    pub search_focused: bool,
    /// Search results (shows)
    pub shows: Vec<Show>,
    /// Selected show
    pub selected_show: Option<Show>,
    /// Episodes for the selected show
    pub episodes: Vec<Episode>,
    /// Current episode
    pub current_episode: Option<Episode>,
    /// Available stream sources
    pub sources: Vec<StreamSource>,
    /// Selected source
    pub selected_source: Option<StreamSource>,
    /// List state for shows
    pub show_list_state: ListState,
    /// List state for episodes
    pub episode_list_state: ListState,
    /// List state for quality selection
    pub quality_list_state: ListState,
    /// Playback menu state
    pub playback_list_state: ListState,
    /// Watch history records for sidebar
    pub history_records: Vec<(String, String, i64, String)>, // (show_id, name, episode, mode)
    /// History list state (for sidebar)
    pub history_list_state: ListState,
    /// Startup menu state
    pub startup_list_state: ListState,
    /// Batch menu state
    pub batch_list_state: ListState,
    /// Loading message
    pub loading_message: String,
    /// Current mode (sub/dub)
    pub mode: String,
    /// Current quality preference
    pub quality: String,
    /// Status message to display
    pub status_message: Option<String>,
    /// Error message to display
    pub error_message: Option<String>,
    /// Whether download mode is enabled
    pub download_mode: bool,
    /// Range input for batch downloads
    pub range_input: String,
    /// Whether we're in range input mode
    pub range_input_mode: bool,
    /// Whether help modal is shown
    pub show_help: bool,
    /// Episode filter input
    pub episode_filter: String,
    /// Whether episode filter is active
    pub episode_filter_active: bool,
}

impl App {
    /// Create a new App with default state.
    pub fn new(mode: String, quality: String, download_mode: bool) -> Self {
        let mut startup_state = ListState::default();
        startup_state.select(Some(0));

        Self {
            screen: Screen::Startup,
            focus: Focus::Main,
            should_quit: false,
            search_input: String::new(),
            search_focused: false,
            shows: Vec::new(),
            selected_show: None,
            episodes: Vec::new(),
            current_episode: None,
            sources: Vec::new(),
            selected_source: None,
            show_list_state: ListState::default(),
            episode_list_state: ListState::default(),
            quality_list_state: ListState::default(),
            playback_list_state: ListState::default(),
            history_records: Vec::new(),
            history_list_state: ListState::default(),
            startup_list_state: startup_state,
            batch_list_state: ListState::default(),
            loading_message: String::new(),
            mode,
            quality,
            status_message: None,
            error_message: None,
            download_mode,
            range_input: String::new(),
            range_input_mode: false,
            show_help: false,
            episode_filter: String::new(),
            episode_filter_active: false,
        }
    }

    /// Get filtered episodes based on current filter.
    pub fn get_filtered_episodes(&self) -> Vec<&Episode> {
        if self.episode_filter.is_empty() {
            self.episodes.iter().collect()
        } else {
            let filter_lower = self.episode_filter.to_lowercase();
            self.episodes
                .iter()
                .filter(|e| {
                    // Match by episode number
                    let num_str = e.number.to_string();
                    if num_str.contains(&self.episode_filter) {
                        return true;
                    }
                    // Match by title if present
                    if let Some(title) = &e.title {
                        if title.to_lowercase().contains(&filter_lower) {
                            return true;
                        }
                    }
                    false
                })
                .collect()
        }
    }

    /// Set the app to loading state with a message.
    pub fn set_loading(&mut self, message: &str) {
        self.screen = Screen::Loading;
        self.loading_message = message.to_string();
    }

    /// Set shows and switch to show list screen.
    pub fn set_shows(&mut self, shows: Vec<Show>) {
        self.shows = shows;
        self.show_list_state.select(Some(0));
        self.screen = Screen::ShowList;
    }

    /// Set episodes and switch to episode list screen.
    pub fn set_episodes(&mut self, episodes: Vec<Episode>) {
        self.episodes = episodes;
        self.episode_list_state.select(Some(0));
        self.screen = Screen::EpisodeList;
    }

    /// Set sources and switch to quality select screen.
    pub fn set_sources(&mut self, sources: Vec<StreamSource>) {
        self.sources = sources;
        self.quality_list_state.select(Some(0));
        self.screen = Screen::QualitySelect;
    }

    /// Set history records for the continue menu.
    pub fn set_history(&mut self, records: Vec<(String, String, i64, String)>) {
        let has_records = !records.is_empty();
        self.history_records = records;
        if has_records {
            self.history_list_state.select(Some(0));
        }
    }

    /// Switch to playback menu.
    pub fn show_playback_menu(&mut self) {
        self.playback_list_state.select(Some(0));
        self.screen = Screen::Playback;
    }

    /// Switch to batch select menu.
    pub fn show_batch_menu(&mut self) {
        self.batch_list_state.select(Some(0));
        self.screen = Screen::BatchSelect;
    }

    /// Set an error message.
    pub fn set_error(&mut self, message: &str) {
        self.error_message = Some(message.to_string());
    }

    /// Clear error message.
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Set status message.
    pub fn set_status(&mut self, message: &str) {
        self.status_message = Some(message.to_string());
    }

    /// Handle keyboard input and return an action.
    pub fn handle_input(&mut self, key: KeyEvent) -> Action {
        // Global quit with Ctrl+C or Ctrl+Q
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') | KeyCode::Char('q') => {
                    self.should_quit = true;
                    return Action::Quit;
                }
                _ => {}
            }
        }

        // Handle help modal
        if self.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
                    self.show_help = false;
                }
                _ => {}
            }
            return Action::None;
        }

        // Toggle help with ?
        if key.code == KeyCode::Char('?') {
            self.show_help = true;
            return Action::None;
        }

        // Handle range input mode specially
        if self.range_input_mode {
            return self.handle_range_input(key);
        }

        // Handle search bar input when focused
        if self.search_focused {
            return self.handle_search_bar_input(key);
        }

        // Handle episode filter input when active
        if self.episode_filter_active {
            return self.handle_episode_filter_input(key);
        }

        // Global keys that work in most screens
        match key.code {
            // Tab to switch focus
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::Sidebar => Focus::Main,
                    Focus::Main => Focus::Sidebar,
                };
                // Initialize sidebar selection if needed
                if self.focus == Focus::Sidebar && self.history_list_state.selected().is_none() && !self.history_records.is_empty() {
                    self.history_list_state.select(Some(0));
                }
                return Action::None;
            }
            // `/` to focus search bar from anywhere
            KeyCode::Char('/') => {
                self.search_focused = true;
                return Action::None;
            }
            _ => {}
        }

        // Handle sidebar input when focused
        if self.focus == Focus::Sidebar {
            return self.handle_sidebar_input(key);
        }

        match self.screen {
            Screen::Startup => self.handle_startup_input(key),
            Screen::Search => self.handle_search_input(key),
            Screen::ShowList => self.handle_show_list_input(key),
            Screen::EpisodeList => self.handle_episode_list_input(key),
            Screen::QualitySelect => self.handle_quality_input(key),
            Screen::Playback => self.handle_playback_input(key),
            Screen::BatchSelect => self.handle_batch_input(key),
            Screen::Loading => {
                // Allow quit during loading
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    self.should_quit = true;
                    return Action::Quit;
                }
                Action::None
            }
        }
    }

    fn handle_search_bar_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Enter => {
                if !self.search_input.is_empty() {
                    let query = self.search_input.clone();
                    self.search_input.clear();
                    self.search_focused = false;
                    self.focus = Focus::Main;
                    Action::Search(query)
                } else {
                    self.search_focused = false;
                    Action::None
                }
            }
            KeyCode::Char(c) => {
                self.search_input.push(c);
                Action::None
            }
            KeyCode::Backspace => {
                self.search_input.pop();
                Action::None
            }
            KeyCode::Esc => {
                self.search_input.clear();
                self.search_focused = false;
                Action::None
            }
            _ => Action::None,
        }
    }

    fn handle_sidebar_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.history_list_state.selected().unwrap_or(0);
                if i > 0 {
                    self.history_list_state.select(Some(i - 1));
                }
                Action::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.history_list_state.selected().unwrap_or(0);
                if i < self.history_records.len().saturating_sub(1) {
                    self.history_list_state.select(Some(i + 1));
                }
                Action::None
            }
            KeyCode::Enter => {
                if let Some(i) = self.history_list_state.selected() {
                    if i < self.history_records.len() {
                        self.focus = Focus::Main;
                        return Action::ContinueFromHistory(i);
                    }
                }
                Action::None
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                Action::Quit
            }
            _ => Action::None,
        }
    }

    fn handle_startup_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.history_records.is_empty() {
                    let i = self.startup_list_state.selected().unwrap_or(0);
                    if i > 0 {
                        self.startup_list_state.select(Some(i - 1));
                    }
                } else {
                    let i = self.history_list_state.selected().unwrap_or(0);
                    if i > 0 {
                        self.history_list_state.select(Some(i - 1));
                    }
                }
                Action::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.history_records.is_empty() {
                    let i = self.startup_list_state.selected().unwrap_or(0);
                    if i < 1 {
                        self.startup_list_state.select(Some(i + 1));
                    }
                } else {
                    let i = self.history_list_state.selected().unwrap_or(0);
                    if i < self.history_records.len().saturating_sub(1) {
                        self.history_list_state.select(Some(i + 1));
                    }
                }
                Action::None
            }
            KeyCode::Enter => {
                if self.history_records.is_empty() {
                    match self.startup_list_state.selected() {
                        Some(0) => Action::NewSearch,
                        _ => Action::NewSearch,
                    }
                } else {
                    if let Some(i) = self.history_list_state.selected() {
                        Action::ContinueFromHistory(i)
                    } else {
                        Action::NewSearch
                    }
                }
            }
            KeyCode::Char('s') | KeyCode::Char('n') => {
                self.screen = Screen::Search;
                Action::None
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                Action::Quit
            }
            _ => Action::None,
        }
    }

    fn handle_search_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Enter => {
                if !self.search_input.is_empty() {
                    let query = self.search_input.clone();
                    self.search_input.clear();
                    Action::Search(query)
                } else {
                    Action::None
                }
            }
            KeyCode::Char(c) => {
                self.search_input.push(c);
                Action::None
            }
            KeyCode::Backspace => {
                self.search_input.pop();
                Action::None
            }
            KeyCode::Esc => {
                self.search_input.clear();
                if !self.shows.is_empty() {
                    self.screen = Screen::ShowList;
                } else {
                    self.screen = Screen::Startup;
                }
                Action::None
            }
            _ => Action::None,
        }
    }

    fn handle_show_list_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.show_list_state.selected().unwrap_or(0);
                if i > 0 {
                    self.show_list_state.select(Some(i - 1));
                }
                Action::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.show_list_state.selected().unwrap_or(0);
                if i < self.shows.len().saturating_sub(1) {
                    self.show_list_state.select(Some(i + 1));
                }
                Action::None
            }
            KeyCode::Enter => {
                if let Some(i) = self.show_list_state.selected() {
                    Action::SelectShow(i)
                } else {
                    Action::None
                }
            }
            KeyCode::Char('s') | KeyCode::Char('/') => {
                self.screen = Screen::Search;
                Action::None
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                Action::Quit
            }
            _ => Action::None,
        }
    }

    fn handle_episode_list_input(&mut self, key: KeyEvent) -> Action {
        let filtered_len = self.get_filtered_episodes().len();

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.episode_list_state.selected().unwrap_or(0);
                if i > 0 {
                    self.episode_list_state.select(Some(i - 1));
                }
                Action::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.episode_list_state.selected().unwrap_or(0);
                if i < filtered_len.saturating_sub(1) {
                    self.episode_list_state.select(Some(i + 1));
                }
                Action::None
            }
            KeyCode::Enter => {
                if let Some(i) = self.episode_list_state.selected() {
                    // Get the actual episode from filtered list
                    let filtered = self.get_filtered_episodes();
                    if i < filtered.len() {
                        let episode_num = filtered[i].number;
                        // Find the index in the original list
                        if let Some(original_idx) = self.episodes.iter().position(|e| e.number == episode_num) {
                            return Action::SelectEpisode(original_idx);
                        }
                    }
                }
                Action::None
            }
            KeyCode::Char('f') => {
                self.episode_filter_active = true;
                Action::None
            }
            KeyCode::Char('s') => {
                self.screen = Screen::Search;
                Action::None
            }
            KeyCode::Backspace | KeyCode::Esc => {
                if !self.episode_filter.is_empty() {
                    // Clear filter first
                    self.episode_filter.clear();
                    self.episode_list_state.select(Some(0));
                } else {
                    self.screen = Screen::ShowList;
                }
                Action::None
            }
            KeyCode::Char('q') => {
                self.should_quit = true;
                Action::Quit
            }
            _ => Action::None,
        }
    }

    fn handle_episode_filter_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => {
                self.episode_filter_active = false;
                // Reset selection to first item after filtering
                if !self.get_filtered_episodes().is_empty() {
                    self.episode_list_state.select(Some(0));
                }
                Action::None
            }
            KeyCode::Char(c) => {
                self.episode_filter.push(c);
                // Reset selection when filter changes
                self.episode_list_state.select(Some(0));
                Action::None
            }
            KeyCode::Backspace => {
                self.episode_filter.pop();
                self.episode_list_state.select(Some(0));
                Action::None
            }
            _ => Action::None,
        }
    }

    fn handle_quality_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.quality_list_state.selected().unwrap_or(0);
                if i > 0 {
                    self.quality_list_state.select(Some(i - 1));
                }
                Action::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.quality_list_state.selected().unwrap_or(0);
                if i < self.sources.len().saturating_sub(1) {
                    self.quality_list_state.select(Some(i + 1));
                }
                Action::None
            }
            KeyCode::Enter => {
                if let Some(i) = self.quality_list_state.selected() {
                    Action::SelectQuality(i)
                } else {
                    Action::None
                }
            }
            KeyCode::Backspace | KeyCode::Esc => {
                self.screen = Screen::EpisodeList;
                Action::None
            }
            KeyCode::Char('q') => {
                self.should_quit = true;
                Action::Quit
            }
            _ => Action::None,
        }
    }

    fn handle_playback_input(&mut self, key: KeyEvent) -> Action {
        let options = self.get_playback_options();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.playback_list_state.selected().unwrap_or(0);
                if i > 0 {
                    self.playback_list_state.select(Some(i - 1));
                }
                Action::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.playback_list_state.selected().unwrap_or(0);
                if i < options.len().saturating_sub(1) {
                    self.playback_list_state.select(Some(i + 1));
                }
                Action::None
            }
            KeyCode::Enter => {
                if let Some(i) = self.playback_list_state.selected() {
                    if i < options.len() {
                        match options[i].as_str() {
                            "Next episode" => Action::Next,
                            "Replay" => Action::Replay,
                            "Previous episode" => Action::Previous,
                            "Select episode" => Action::BackToEpisodes,
                            "Quit" => {
                                self.should_quit = true;
                                Action::Quit
                            }
                            _ => Action::None,
                        }
                    } else {
                        Action::None
                    }
                } else {
                    Action::None
                }
            }
            KeyCode::Char('n') => Action::Next,
            KeyCode::Char('p') => Action::Previous,
            KeyCode::Char('r') => Action::Replay,
            KeyCode::Char('e') => Action::BackToEpisodes,
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                Action::Quit
            }
            _ => Action::None,
        }
    }

    fn handle_batch_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.batch_list_state.selected().unwrap_or(0);
                if i > 0 {
                    self.batch_list_state.select(Some(i - 1));
                }
                Action::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.batch_list_state.selected().unwrap_or(0);
                if i < 2 {
                    self.batch_list_state.select(Some(i + 1));
                }
                Action::None
            }
            KeyCode::Enter => {
                match self.batch_list_state.selected() {
                    Some(0) => Action::BatchAll,
                    Some(1) => {
                        self.range_input_mode = true;
                        self.range_input.clear();
                        Action::None
                    }
                    Some(2) => Action::BatchSingle,
                    _ => Action::None,
                }
            }
            KeyCode::Backspace | KeyCode::Esc => {
                self.screen = Screen::EpisodeList;
                Action::None
            }
            KeyCode::Char('q') => {
                self.should_quit = true;
                Action::Quit
            }
            _ => Action::None,
        }
    }

    fn handle_range_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Enter => {
                let parts: Vec<&str> = self.range_input.split('-').collect();
                if parts.len() == 2 {
                    if let (Ok(start), Ok(end)) = (parts[0].trim().parse(), parts[1].trim().parse()) {
                        self.range_input_mode = false;
                        return Action::BatchRange(start, end);
                    }
                }
                self.set_error("Invalid range format. Use: start-end (e.g., 1-12)");
                Action::None
            }
            KeyCode::Char(c) if c.is_ascii_digit() || c == '-' => {
                self.range_input.push(c);
                Action::None
            }
            KeyCode::Backspace => {
                self.range_input.pop();
                Action::None
            }
            KeyCode::Esc => {
                self.range_input_mode = false;
                self.range_input.clear();
                Action::None
            }
            _ => Action::None,
        }
    }

    /// Get playback options based on current state.
    fn get_playback_options(&self) -> Vec<String> {
        let mut options = Vec::new();

        if let Some(ep) = &self.current_episode {
            let current_idx = self.episodes.iter().position(|e| e.number == ep.number);
            let has_next = current_idx.map(|i| i + 1 < self.episodes.len()).unwrap_or(false);
            let has_prev = current_idx.map(|i| i > 0).unwrap_or(false);

            if has_next {
                options.push("Next episode".to_string());
            }
            options.push("Replay".to_string());
            if has_prev {
                options.push("Previous episode".to_string());
            }
        }

        options.push("Select episode".to_string());
        options.push("Quit".to_string());

        options
    }
}

/// Draw the UI.
pub fn draw(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(3),  // Search bar
            Constraint::Min(0),     // Content (sidebar + main)
            Constraint::Length(3),  // Footer
        ])
        .split(size);

    // Draw header
    draw_header(frame, app, chunks[0]);

    // Draw search bar
    draw_search_bar(frame, app, chunks[1]);

    // Split content area into sidebar and main
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(30),  // Sidebar (fixed width)
            Constraint::Min(0),      // Main content
        ])
        .split(chunks[2]);

    // Draw sidebar with history
    draw_sidebar(frame, app, content_chunks[0]);

    // Draw main content based on screen
    match app.screen {
        Screen::Loading => draw_loading(frame, app, content_chunks[1]),
        Screen::Search => draw_search_help(frame, app, content_chunks[1]),
        Screen::Startup => draw_startup_main(frame, app, content_chunks[1]),
        Screen::ShowList => draw_show_list_main(frame, app, content_chunks[1]),
        Screen::EpisodeList => draw_episode_list_main(frame, app, content_chunks[1]),
        Screen::QualitySelect => draw_quality_select(frame, app, content_chunks[1]),
        Screen::Playback => draw_playback(frame, app, content_chunks[1]),
        Screen::BatchSelect => draw_batch_select(frame, app, content_chunks[1]),
    }

    // Draw footer
    draw_footer(frame, app, chunks[3]);

    // Draw error popup if there's an error
    if let Some(error) = &app.error_message {
        draw_error_popup(frame, error);
    }

    // Draw range input popup if in range input mode
    if app.range_input_mode {
        draw_range_input_popup(frame, &app.range_input);
    }

    // Draw help modal if shown
    if app.show_help {
        draw_help_modal(frame, app);
    }
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let mode_style = if app.mode == "dub" {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Cyan)
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled("anime-watcher", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(format!("[{}]", app.mode), mode_style),
        Span::raw("  "),
        Span::styled(format!("[{}]", app.quality), Style::default().fg(Color::Green)),
        if app.download_mode {
            Span::styled("  [download]", Style::default().fg(Color::Red))
        } else {
            Span::raw("")
        },
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(header, area);
}

fn draw_search_bar(frame: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.search_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let search_text = if app.search_input.is_empty() && !app.search_focused {
        "Press '/' to search..."
    } else {
        &app.search_input
    };

    let search = Paragraph::new(search_text)
        .style(if app.search_focused {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search")
                .border_style(border_style),
        );

    frame.render_widget(search, area);

    // Show cursor if search is focused
    if app.search_focused {
        frame.set_cursor_position((
            area.x + app.search_input.len() as u16 + 1,
            area.y + 1,
        ));
    }
}

fn draw_sidebar(frame: &mut Frame, app: &mut App, area: Rect) {
    let border_style = if app.focus == Focus::Sidebar {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    if app.history_records.is_empty() {
        let empty = Paragraph::new("No watch history")
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Recent")
                    .border_style(border_style),
            );
        frame.render_widget(empty, area);
    } else {
        let items: Vec<ListItem> = app
            .history_records
            .iter()
            .map(|(_, name, ep, _)| {
                // Truncate name if too long
                let display_name = if name.len() > 20 {
                    format!("{}...", &name[..17])
                } else {
                    name.clone()
                };
                ListItem::new(format!("{} [{}]", display_name, ep))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Recent")
                    .border_style(border_style),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, &mut app.history_list_state);
    }
}

fn draw_search_help(frame: &mut Frame, _app: &App, area: Rect) {
    let help = Paragraph::new("Type your search query and press Enter\n\nPress Esc to cancel")
        .block(Block::default().borders(Borders::ALL).title("Search"))
        .wrap(Wrap { trim: true });

    frame.render_widget(help, area);
}

fn draw_startup_main(frame: &mut Frame, _app: &App, area: Rect) {
    let welcome = Paragraph::new(
        "Welcome to anime-watcher!\n\n\
        - Press '/' to search for anime\n\
        - Select from Recent history on the left\n\
        - Use Tab to switch between panels\n\n\
        Keyboard shortcuts:\n\
        - j/k or arrows: Navigate\n\
        - Enter: Select\n\
        - Tab: Switch panel\n\
        - q: Quit"
    )
    .block(Block::default().borders(Borders::ALL).title("Welcome"))
    .wrap(Wrap { trim: true });

    frame.render_widget(welcome, area);
}

fn draw_show_list_main(frame: &mut Frame, app: &mut App, area: Rect) {
    // Split into list and details
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Show list
    let items: Vec<ListItem> = app
        .shows
        .iter()
        .map(|s| ListItem::new(s.to_display()))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Search Results"))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, chunks[0], &mut app.show_list_state);

    // Show details
    let details = if let Some(i) = app.show_list_state.selected() {
        if i < app.shows.len() {
            let show = &app.shows[i];
            format!(
                "Name: {}\nEpisodes: {}\n\nPress Enter to view episodes",
                show.name, show.available_episodes
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let details_widget = Paragraph::new(details)
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .wrap(Wrap { trim: true });

    frame.render_widget(details_widget, chunks[1]);
}

fn draw_episode_list_main(frame: &mut Frame, app: &mut App, area: Rect) {
    // Determine layout based on whether filter is active
    let chunks = if app.episode_filter_active || !app.episode_filter.is_empty() {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),      // Filter input
                Constraint::Percentage(60), // Episode list
                Constraint::Percentage(30), // Details
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(0),      // No filter input
                Constraint::Percentage(70), // Episode list
                Constraint::Percentage(30), // Details
            ])
            .split(area)
    };

    // Draw filter input if active or has content
    if app.episode_filter_active || !app.episode_filter.is_empty() {
        let filter_style = if app.episode_filter_active {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let filter_title = if app.episode_filter.is_empty() {
            "Filter (type to search)".to_string()
        } else {
            format!("Filter ({} matches)", app.get_filtered_episodes().len())
        };

        let filter_input = Paragraph::new(app.episode_filter.as_str())
            .style(filter_style)
            .block(Block::default().borders(Borders::ALL).title(filter_title));

        frame.render_widget(filter_input, chunks[0]);

        // Show cursor when filter is active
        if app.episode_filter_active {
            frame.set_cursor_position((
                chunks[0].x + app.episode_filter.len() as u16 + 1,
                chunks[0].y + 1,
            ));
        }
    }

    // Get filtered episodes
    let filtered_episodes = app.get_filtered_episodes();

    // Episode list using filtered episodes
    let items: Vec<ListItem> = filtered_episodes
        .iter()
        .map(|e| ListItem::new(e.to_display()))
        .collect();

    let title = if let Some(show) = &app.selected_show {
        if !app.episode_filter.is_empty() {
            format!("{} (filtered)", show.name)
        } else {
            show.name.clone()
        }
    } else {
        "Episodes".to_string()
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, chunks[1], &mut app.episode_list_state);

    // Episode details from filtered list
    let details = if let Some(i) = app.episode_list_state.selected() {
        let filtered = app.get_filtered_episodes();
        if i < filtered.len() {
            let episode = filtered[i];
            let action = if app.download_mode { "download" } else { "stream" };
            format!("Episode {}\n\nPress Enter to {}", episode.number, action)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let details_widget = Paragraph::new(details)
        .block(Block::default().borders(Borders::ALL).title("Info"))
        .wrap(Wrap { trim: true });

    frame.render_widget(details_widget, chunks[2]);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = if app.search_focused {
        "[Enter] search  [Esc] cancel  [?] help"
    } else {
        match app.screen {
            Screen::Startup => "[/] search  [Tab] switch  [↑↓] navigate  [Enter] select  [?] help  [q] quit",
            Screen::Search => "[/] search  [Tab] switch  [?] help  [q] quit",
            Screen::ShowList => "[/] search  [Tab] switch  [↑↓] navigate  [Enter] select  [?] help  [q] quit",
            Screen::EpisodeList => "[/] search  [Tab] switch  [↑↓] navigate  [f] filter  [Enter] select  [?] help  [q] quit",
            Screen::QualitySelect => "[↑↓] navigate  [Enter] select  [Bksp] back  [?] help  [q] quit",
            Screen::Playback => "[/] search  [Tab] switch  [n] next  [p] prev  [r] replay  [?] help  [q] quit",
            Screen::BatchSelect => "[↑↓] navigate  [Enter] select  [Bksp] back  [?] help  [q] quit",
            Screen::Loading => "[?] help  [q] quit",
        }
    };

    let footer = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, area);
}

fn draw_loading(frame: &mut Frame, app: &App, area: Rect) {
    let loading = Paragraph::new(app.loading_message.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Loading"));

    frame.render_widget(loading, area);
}

fn draw_search(frame: &mut Frame, app: &App, area: Rect) {
    let input = Paragraph::new(app.search_input.as_str())
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("Search Anime"));

    frame.render_widget(input, area);

    // Show cursor
    frame.set_cursor_position((
        area.x + app.search_input.len() as u16 + 1,
        area.y + 1,
    ));
}

fn draw_startup(frame: &mut Frame, app: &mut App, area: Rect) {
    if app.history_records.is_empty() {
        let items: Vec<ListItem> = vec![
            ListItem::new("New search"),
        ];

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Welcome"))
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, &mut app.startup_list_state);
    } else {
        let items: Vec<ListItem> = app
            .history_records
            .iter()
            .map(|(_, name, ep, mode)| {
                ListItem::new(format!("{} - Episode {} [{}]", name, ep, mode))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Continue Watching (Press 's' for new search)"))
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, &mut app.history_list_state);
    }
}

fn draw_show_list(frame: &mut Frame, app: &mut App, area: Rect) {
    // Split into sidebar and main
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Show list (sidebar)
    let items: Vec<ListItem> = app
        .shows
        .iter()
        .map(|s| ListItem::new(s.to_display()))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Search Results"))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, chunks[0], &mut app.show_list_state);

    // Show details (main panel)
    let details = if let Some(i) = app.show_list_state.selected() {
        if i < app.shows.len() {
            let show = &app.shows[i];
            format!(
                "Name: {}\n\nEpisodes: {}\n\nPress Enter to view episodes",
                show.name, show.available_episodes
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let details_widget = Paragraph::new(details)
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .wrap(Wrap { trim: true });

    frame.render_widget(details_widget, chunks[1]);
}

fn draw_episode_list(frame: &mut Frame, app: &mut App, area: Rect) {
    // Split into sidebar and main
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Episode list (sidebar)
    let items: Vec<ListItem> = app
        .episodes
        .iter()
        .map(|e| ListItem::new(e.to_display()))
        .collect();

    let title = if let Some(show) = &app.selected_show {
        format!("Episodes - {}", show.name)
    } else {
        "Episodes".to_string()
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, chunks[0], &mut app.episode_list_state);

    // Episode details (main panel)
    let details = if let Some(i) = app.episode_list_state.selected() {
        if i < app.episodes.len() {
            let episode = &app.episodes[i];
            let action = if app.download_mode { "download" } else { "stream" };
            format!(
                "Episode {}\n\nPress Enter to {}",
                episode.number, action
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let details_widget = Paragraph::new(details)
        .block(Block::default().borders(Borders::ALL).title("Episode Info"))
        .wrap(Wrap { trim: true });

    frame.render_widget(details_widget, chunks[1]);
}

fn draw_quality_select(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .sources
        .iter()
        .map(|s| ListItem::new(s.to_display()))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Select Quality"))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut app.quality_list_state);
}

fn draw_playback(frame: &mut Frame, app: &mut App, area: Rect) {
    let options = app.get_playback_options();
    let items: Vec<ListItem> = options.iter().map(|s| ListItem::new(s.as_str())).collect();

    let title = if let Some(ep) = &app.current_episode {
        format!("Episode {} - What next?", ep.number)
    } else {
        "What next?".to_string()
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut app.playback_list_state);
}

fn draw_batch_select(frame: &mut Frame, app: &mut App, area: Rect) {
    let items = vec![
        ListItem::new("All episodes"),
        ListItem::new("Range (e.g., 1-12)"),
        ListItem::new("Single episode"),
    ];

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Download Mode"))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut app.batch_list_state);
}

fn draw_error_popup(frame: &mut Frame, error: &str) {
    let area = centered_rect(60, 20, frame.area());
    frame.render_widget(Clear, area);

    let popup = Paragraph::new(error)
        .style(Style::default().fg(Color::Red))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Error")
                .border_style(Style::default().fg(Color::Red)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(popup, area);
}

fn draw_range_input_popup(frame: &mut Frame, input: &str) {
    let area = centered_rect(50, 15, frame.area());
    frame.render_widget(Clear, area);

    let popup = Paragraph::new(format!("Enter range (e.g., 1-12): {}", input))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Episode Range"),
        );

    frame.render_widget(popup, area);

    // Show cursor
    frame.set_cursor_position((
        area.x + 26 + input.len() as u16,
        area.y + 1,
    ));
}

fn draw_help_modal(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 80, frame.area());
    frame.render_widget(Clear, area);

    // Get context-sensitive help content
    let (title, content) = get_help_content(app);

    let help_text = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Help - {}", title))
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(help_text, area);
}

fn get_help_content(app: &App) -> (&'static str, String) {
    let global_keys = "\
Global Commands
───────────────
  ?           Show/hide this help
  Ctrl+C      Force quit
  Ctrl+Q      Force quit
  /           Focus search bar
  Tab         Switch panel focus
  q           Quit

";

    let search_keys = "\
Search Bar
──────────
  Enter       Execute search
  Esc         Cancel search
  Backspace   Delete character

";

    let navigation_keys = "\
Navigation
──────────
  j / ↓       Move down
  k / ↑       Move up
  Enter       Select item
  Backspace   Go back

";

    let sidebar_keys = "\
Sidebar (Recent)
────────────────
  j / ↓       Move down
  k / ↑       Move up
  Enter       Load anime from history
  Tab         Switch to main panel

";

    let playback_keys = "\
Playback Controls
─────────────────
  n           Next episode
  p           Previous episode
  r           Replay current
  e           Back to episode list
  Tab         Switch to sidebar

";

    let batch_keys = "\
Batch Download
──────────────
  All         Download all episodes
  Range       Download range (e.g., 1-12)
  Single      Download selected episode only

";

    let filter_keys = "\
Episode Filter
──────────────
  f           Activate filter
  Enter       Confirm filter
  Esc         Cancel filter
  Backspace   Delete character
  (Type)      Filter by episode number/title

";

    match app.screen {
        Screen::Startup => {
            let content = format!(
                "{}{}{}Press ? to close",
                global_keys, sidebar_keys, search_keys
            );
            ("Startup", content)
        }
        Screen::Search => {
            let content = format!(
                "{}{}Press ? to close",
                global_keys, search_keys
            );
            ("Search", content)
        }
        Screen::ShowList => {
            let content = format!(
                "{}{}{}{}Press ? to close",
                global_keys, navigation_keys, sidebar_keys, search_keys
            );
            ("Show List", content)
        }
        Screen::EpisodeList => {
            let content = format!(
                "{}{}{}{}{}Press ? to close",
                global_keys, navigation_keys, filter_keys, sidebar_keys, search_keys
            );
            ("Episode List", content)
        }
        Screen::QualitySelect => {
            let content = format!(
                "{}{}Press ? to close",
                global_keys, navigation_keys
            );
            ("Quality Select", content)
        }
        Screen::Playback => {
            let content = format!(
                "{}{}{}{}Press ? to close",
                global_keys, playback_keys, sidebar_keys, search_keys
            );
            ("Playback", content)
        }
        Screen::BatchSelect => {
            let content = format!(
                "{}{}{}Press ? to close",
                global_keys, navigation_keys, batch_keys
            );
            ("Batch Download", content)
        }
        Screen::Loading => {
            let content = format!(
                "{}Press ? to close",
                global_keys
            );
            ("Loading", content)
        }
    }
}

/// Helper function to create a centered rect.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Poll for keyboard events with a timeout.
pub fn poll_event(timeout: Duration) -> io::Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}
