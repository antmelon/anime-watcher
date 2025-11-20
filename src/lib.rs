//! A command-line anime streaming application written in Rust.
//!
//! anime-watcher is a CLI tool for searching, selecting, and streaming anime episodes
//! from AllAnime. It provides a fuzzy-search interface using fzf and plays content
//! through mpv (or platform-specific players).
//!
//! # Features
//!
//! - Search for anime by name
//! - Browse available episodes
//! - Stream episodes with quality selection
//! - Navigate between episodes without restarting
//! - Support for both subbed and dubbed content
//!
//! # Usage
//!
//! ```bash
//! # Run with default settings (sub mode)
//! cargo run
//!
//! # Run in dub mode
//! cargo run -- -m dub
//! ```

pub mod api;
pub mod config;
pub mod download;
pub mod history;
pub mod tui;
pub mod types;
pub mod ui;
