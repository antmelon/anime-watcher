//! Application state management and input handling.

use crate::config::{ColorScheme, Keybindings};
use crate::types::{Episode, Show, StreamSource};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;

use super::types::{Action, Focus, Screen};

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
    /// Status message to display (reserved for future use)
    #[allow(dead_code)]
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
    /// Whether we're showing batch confirmation dialog
    pub batch_confirm_mode: bool,
    /// The pending batch action to confirm
    pub pending_batch_action: Option<Action>,
    /// Custom keybindings
    pub keybindings: Keybindings,
    /// Color scheme
    pub colors: ColorScheme,
    /// Whether download modal is shown
    pub show_download_modal: bool,
    /// Current download index (1-based)
    pub download_current: usize,
    /// Total downloads in batch
    pub download_total: usize,
    /// Current download message
    pub download_message: String,
    /// Download activity log
    pub download_log: Vec<String>,
}

impl App {
    /// Create a new App with default state.
    pub fn new(
        mode: String,
        quality: String,
        download_mode: bool,
        keybindings: Keybindings,
        colors: ColorScheme,
    ) -> Self {
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
            batch_confirm_mode: false,
            pending_batch_action: None,
            keybindings,
            colors,
            show_download_modal: false,
            download_current: 0,
            download_total: 0,
            download_message: String::new(),
            download_log: Vec::new(),
        }
    }

    /// Initialize and show the download modal for batch downloads.
    ///
    /// This resets all download state and displays the modal overlay.
    ///
    /// # Arguments
    ///
    /// * `total` - Total number of episodes to download
    pub fn start_download_modal(&mut self, total: usize) {
        self.show_download_modal = true;
        self.download_current = 0;
        self.download_total = total;
        self.download_message = String::new();
        self.download_log.clear();
    }

    /// Update the current download progress displayed in the modal.
    ///
    /// # Arguments
    ///
    /// * `current` - Current episode index (1-based)
    /// * `message` - Status message to display (e.g., "Downloading Episode 5...")
    pub fn update_download_progress(&mut self, current: usize, message: &str) {
        self.download_current = current;
        self.download_message = message.to_string();
    }

    /// Add an entry to the download activity log.
    ///
    /// The log is limited to the 10 most recent entries to prevent overflow.
    ///
    /// # Arguments
    ///
    /// * `entry` - Log entry to add (e.g., "✓ Ep 5 complete")
    pub fn add_download_log(&mut self, entry: &str) {
        self.download_log.push(entry.to_string());
        // Keep only last 10 entries to avoid overflow
        if self.download_log.len() > 10 {
            self.download_log.remove(0);
        }
    }

    /// Close the download modal and return to normal view.
    pub fn close_download_modal(&mut self) {
        self.show_download_modal = false;
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

    /// Set sources and switch to quality select screen (reserved for future use).
    #[allow(dead_code)]
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

    /// Set status message (reserved for future use).
    #[allow(dead_code)]
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
            if key.code == KeyCode::Esc
                || self.keybindings.matches(&self.keybindings.help, &key)
                || self.keybindings.matches(&self.keybindings.quit, &key)
            {
                self.show_help = false;
            }
            return Action::None;
        }

        // Toggle help
        if self.keybindings.matches(&self.keybindings.help, &key) {
            self.show_help = true;
            return Action::None;
        }

        // Handle range input mode specially
        if self.range_input_mode {
            return self.handle_range_input(key);
        }

        // Handle batch confirmation mode
        if self.batch_confirm_mode {
            return self.handle_batch_confirm(key);
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
        // Toggle focus
        if self
            .keybindings
            .matches(&self.keybindings.toggle_focus, &key)
        {
            self.focus = match self.focus {
                Focus::Sidebar => Focus::Main,
                Focus::Main => Focus::Sidebar,
            };
            // Initialize sidebar selection if needed
            if self.focus == Focus::Sidebar
                && self.history_list_state.selected().is_none()
                && !self.history_records.is_empty()
            {
                self.history_list_state.select(Some(0));
            }
            return Action::None;
        }

        // Focus search bar from anywhere
        if self.keybindings.matches(&self.keybindings.search, &key) {
            self.search_focused = true;
            return Action::None;
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
                if self.keybindings.matches(&self.keybindings.quit, &key) {
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
        if self.keybindings.matches(&self.keybindings.up, &key) {
            let i = self.history_list_state.selected().unwrap_or(0);
            if i > 0 {
                self.history_list_state.select(Some(i - 1));
            }
            Action::None
        } else if self.keybindings.matches(&self.keybindings.down, &key) {
            let i = self.history_list_state.selected().unwrap_or(0);
            if i < self.history_records.len().saturating_sub(1) {
                self.history_list_state.select(Some(i + 1));
            }
            Action::None
        } else if self.keybindings.matches(&self.keybindings.select, &key) {
            if let Some(i) = self.history_list_state.selected() {
                if i < self.history_records.len() {
                    self.focus = Focus::Main;
                    return Action::ContinueFromHistory(i);
                }
            }
            Action::None
        } else if self.keybindings.matches(&self.keybindings.quit, &key) {
            self.should_quit = true;
            Action::Quit
        } else {
            Action::None
        }
    }

    fn handle_startup_input(&mut self, key: KeyEvent) -> Action {
        if self.keybindings.matches(&self.keybindings.up, &key) {
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
        } else if self.keybindings.matches(&self.keybindings.down, &key) {
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
        } else if self.keybindings.matches(&self.keybindings.select, &key) {
            if self.history_records.is_empty() {
                match self.startup_list_state.selected() {
                    Some(0) => Action::NewSearch,
                    _ => Action::NewSearch,
                }
            } else if let Some(i) = self.history_list_state.selected() {
                Action::ContinueFromHistory(i)
            } else {
                Action::NewSearch
            }
        } else if self.keybindings.matches(&self.keybindings.new_search, &key) {
            self.screen = Screen::Search;
            Action::None
        } else if self.keybindings.matches(&self.keybindings.quit, &key) {
            self.should_quit = true;
            Action::Quit
        } else {
            Action::None
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
        if self.keybindings.matches(&self.keybindings.up, &key) {
            let i = self.show_list_state.selected().unwrap_or(0);
            if i > 0 {
                self.show_list_state.select(Some(i - 1));
            }
            Action::None
        } else if self.keybindings.matches(&self.keybindings.down, &key) {
            let i = self.show_list_state.selected().unwrap_or(0);
            if i < self.shows.len().saturating_sub(1) {
                self.show_list_state.select(Some(i + 1));
            }
            Action::None
        } else if self.keybindings.matches(&self.keybindings.select, &key) {
            if let Some(i) = self.show_list_state.selected() {
                Action::SelectShow(i)
            } else {
                Action::None
            }
        } else if self.keybindings.matches(&self.keybindings.search, &key) {
            self.screen = Screen::Search;
            Action::None
        } else if self.keybindings.matches(&self.keybindings.quit, &key) {
            self.should_quit = true;
            Action::Quit
        } else {
            Action::None
        }
    }

    fn handle_episode_list_input(&mut self, key: KeyEvent) -> Action {
        let filtered_len = self.get_filtered_episodes().len();

        if self.keybindings.matches(&self.keybindings.up, &key) {
            let i = self.episode_list_state.selected().unwrap_or(0);
            if i > 0 {
                self.episode_list_state.select(Some(i - 1));
            }
            Action::None
        } else if self.keybindings.matches(&self.keybindings.down, &key) {
            let i = self.episode_list_state.selected().unwrap_or(0);
            if i < filtered_len.saturating_sub(1) {
                self.episode_list_state.select(Some(i + 1));
            }
            Action::None
        } else if self.keybindings.matches(&self.keybindings.select, &key) {
            if let Some(i) = self.episode_list_state.selected() {
                // Get the actual episode from filtered list
                let filtered = self.get_filtered_episodes();
                if i < filtered.len() {
                    let episode_num = filtered[i].number;
                    // Find the index in the original list
                    if let Some(original_idx) =
                        self.episodes.iter().position(|e| e.number == episode_num)
                    {
                        return Action::SelectEpisode(original_idx);
                    }
                }
            }
            Action::None
        } else if self.keybindings.matches(&self.keybindings.filter, &key) {
            self.episode_filter_active = true;
            Action::None
        } else if self.keybindings.matches(&self.keybindings.search, &key) {
            self.screen = Screen::Search;
            Action::None
        } else if self.keybindings.matches(&self.keybindings.back, &key) {
            if !self.episode_filter.is_empty() {
                // Clear filter first
                self.episode_filter.clear();
                self.episode_list_state.select(Some(0));
            } else {
                self.screen = Screen::ShowList;
            }
            Action::None
        } else if self.keybindings.matches(&self.keybindings.quit, &key) {
            self.should_quit = true;
            Action::Quit
        } else {
            Action::None
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
        if self.keybindings.matches(&self.keybindings.up, &key) {
            let i = self.quality_list_state.selected().unwrap_or(0);
            if i > 0 {
                self.quality_list_state.select(Some(i - 1));
            }
            Action::None
        } else if self.keybindings.matches(&self.keybindings.down, &key) {
            let i = self.quality_list_state.selected().unwrap_or(0);
            if i < self.sources.len().saturating_sub(1) {
                self.quality_list_state.select(Some(i + 1));
            }
            Action::None
        } else if self.keybindings.matches(&self.keybindings.select, &key) {
            if let Some(i) = self.quality_list_state.selected() {
                Action::SelectQuality(i)
            } else {
                Action::None
            }
        } else if self.keybindings.matches(&self.keybindings.back, &key) {
            self.screen = Screen::EpisodeList;
            Action::None
        } else if self.keybindings.matches(&self.keybindings.quit, &key) {
            self.should_quit = true;
            Action::Quit
        } else {
            Action::None
        }
    }

    fn handle_playback_input(&mut self, key: KeyEvent) -> Action {
        let options = self.get_playback_options();

        if self.keybindings.matches(&self.keybindings.up, &key) {
            let i = self.playback_list_state.selected().unwrap_or(0);
            if i > 0 {
                self.playback_list_state.select(Some(i - 1));
            }
            Action::None
        } else if self.keybindings.matches(&self.keybindings.down, &key) {
            let i = self.playback_list_state.selected().unwrap_or(0);
            if i < options.len().saturating_sub(1) {
                self.playback_list_state.select(Some(i + 1));
            }
            Action::None
        } else if self.keybindings.matches(&self.keybindings.select, &key) {
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
        } else if self.keybindings.matches(&self.keybindings.next, &key) {
            Action::Next
        } else if self.keybindings.matches(&self.keybindings.previous, &key) {
            Action::Previous
        } else if self.keybindings.matches(&self.keybindings.replay, &key) {
            Action::Replay
        } else if self.keybindings.matches(&self.keybindings.episodes, &key) {
            Action::BackToEpisodes
        } else if self.keybindings.matches(&self.keybindings.quit, &key) {
            self.should_quit = true;
            Action::Quit
        } else {
            Action::None
        }
    }

    fn handle_batch_input(&mut self, key: KeyEvent) -> Action {
        if self.keybindings.matches(&self.keybindings.up, &key) {
            let i = self.batch_list_state.selected().unwrap_or(0);
            if i > 0 {
                self.batch_list_state.select(Some(i - 1));
            }
            Action::None
        } else if self.keybindings.matches(&self.keybindings.down, &key) {
            let i = self.batch_list_state.selected().unwrap_or(0);
            if i < 2 {
                self.batch_list_state.select(Some(i + 1));
            }
            Action::None
        } else if self.keybindings.matches(&self.keybindings.select, &key) {
            match self.batch_list_state.selected() {
                Some(0) => {
                    // Show confirmation for all episodes
                    self.pending_batch_action = Some(Action::BatchAll);
                    self.batch_confirm_mode = true;
                    Action::None
                }
                Some(1) => {
                    self.range_input_mode = true;
                    self.range_input.clear();
                    Action::None
                }
                Some(2) => Action::BatchSingle,
                _ => Action::None,
            }
        } else if self.keybindings.matches(&self.keybindings.back, &key) {
            self.screen = Screen::EpisodeList;
            Action::None
        } else if self.keybindings.matches(&self.keybindings.quit, &key) {
            self.should_quit = true;
            Action::Quit
        } else {
            Action::None
        }
    }

    fn handle_range_input(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Enter => {
                let parts: Vec<&str> = self.range_input.split('-').collect();
                if parts.len() == 2 {
                    if let (Ok(start), Ok(end)) = (
                        parts[0].trim().parse::<i64>(),
                        parts[1].trim().parse::<i64>(),
                    ) {
                        // Validate range bounds
                        if start > end {
                            self.set_error("Invalid range: start must be <= end");
                            return Action::None;
                        }
                        if start < 1 {
                            self.set_error("Invalid range: start must be >= 1");
                            return Action::None;
                        }

                        // Check against available episodes
                        let max_episode = self.episodes.iter().map(|e| e.number).max().unwrap_or(0);
                        let min_episode = self.episodes.iter().map(|e| e.number).min().unwrap_or(1);

                        if start > max_episode || end > max_episode {
                            self.set_error(&format!(
                                "Invalid range: episodes only go up to {}",
                                max_episode
                            ));
                            return Action::None;
                        }
                        if start < min_episode {
                            self.set_error(&format!(
                                "Invalid range: episodes start at {}",
                                min_episode
                            ));
                            return Action::None;
                        }

                        self.range_input_mode = false;
                        // Show confirmation for range
                        self.pending_batch_action = Some(Action::BatchRange(start, end));
                        self.batch_confirm_mode = true;
                        return Action::None;
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

    fn handle_batch_confirm(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                self.batch_confirm_mode = false;
                if let Some(action) = self.pending_batch_action.take() {
                    action
                } else {
                    Action::None
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.batch_confirm_mode = false;
                self.pending_batch_action = None;
                Action::None
            }
            _ => Action::None,
        }
    }

    /// Get the episode count for pending batch action.
    pub fn get_pending_batch_count(&self) -> usize {
        match &self.pending_batch_action {
            Some(Action::BatchAll) => self.episodes.len(),
            Some(Action::BatchRange(start, end)) => self
                .episodes
                .iter()
                .filter(|e| e.number >= *start && e.number <= *end)
                .count(),
            _ => 0,
        }
    }

    /// Get playback options based on current state.
    pub fn get_playback_options(&self) -> Vec<String> {
        let mut options = Vec::new();

        if let Some(ep) = &self.current_episode {
            let current_idx = self.episodes.iter().position(|e| e.number == ep.number);
            let has_next = current_idx
                .map(|i| i + 1 < self.episodes.len())
                .unwrap_or(false);
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
