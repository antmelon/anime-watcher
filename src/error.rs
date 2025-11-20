//! Custom error types for anime-watcher.
//!
//! This module provides structured error handling instead of String errors.

use std::error::Error;
use std::fmt;
use std::io;

/// Application error types.
#[derive(Debug)]
pub enum AppError {
    /// Network/HTTP errors
    Network(String),
    /// API response parsing errors
    Parse(String),
    /// Configuration errors
    Config(String),
    /// File I/O errors
    Io(io::Error),
    /// Download errors
    Download(String),
    /// No results found
    NotFound(String),
    /// Invalid input from user
    InvalidInput(String),
    /// Player not found or failed to start
    Player(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Network(msg) => write!(f, "Network error: {}", msg),
            AppError::Parse(msg) => write!(f, "Parse error: {}", msg),
            AppError::Config(msg) => write!(f, "Config error: {}", msg),
            AppError::Io(err) => write!(f, "I/O error: {}", err),
            AppError::Download(msg) => write!(f, "Download error: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            AppError::Player(msg) => write!(f, "Player error: {}", msg),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Network(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Parse(err.to_string())
    }
}

impl From<toml::de::Error> for AppError {
    fn from(err: toml::de::Error) -> Self {
        AppError::Config(err.to_string())
    }
}

/// Result type alias using AppError.
pub type Result<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AppError::Network("connection refused".to_string());
        assert_eq!(err.to_string(), "Network error: connection refused");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let app_err: AppError = io_err.into();
        assert!(matches!(app_err, AppError::Io(_)));
    }

    #[test]
    fn test_error_not_found() {
        let err = AppError::NotFound("No episodes found".to_string());
        assert!(err.to_string().contains("No episodes found"));
    }
}
