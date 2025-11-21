//! Terminal User Interface for anime-watcher using ratatui.
//!
//! This module provides a full-screen TUI with panels for browsing
//! and selecting anime shows and episodes.

mod render;
mod state;
mod types;

pub use render::draw;
pub use state::App;
pub use types::{Action, Screen};

use crossterm::event::{self, Event};
use std::io;
use std::time::Duration;

/// Poll for keyboard events with a timeout.
pub fn poll_event(timeout: Duration) -> io::Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}
