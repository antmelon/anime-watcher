//! Download functionality for saving anime episodes to disk.
//!
//! This module provides functions for downloading video files using yt-dlp.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Download a video from a URL using yt-dlp.
///
/// Uses yt-dlp to handle video extraction and downloading, which properly
/// handles HLS streams, embed pages, and other video formats.
///
/// # Arguments
///
/// * `url` - The URL to download from
/// * `output_path` - The path where the file should be saved
///
/// # Returns
///
/// Ok(()) on success, or an error if the download fails.
pub async fn download_file(
    url: &str,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_str = output_path.to_string_lossy();

    // Use yt-dlp for downloading - it handles extraction properly
    let status = Command::new("yt-dlp")
        .arg("--no-warnings")
        .arg("--no-check-certificate")
        .arg("-o")
        .arg(output_str.as_ref())
        .arg("--merge-output-format")
        .arg("mp4")
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "yt-dlp not found. Please install it: https://github.com/yt-dlp/yt-dlp".to_string()
            } else {
                format!("Failed to run yt-dlp: {}", e)
            }
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("yt-dlp exited with status: {}", status.code().unwrap_or(-1)).into())
    }
}

/// Generate a safe filename for an episode.
///
/// # Arguments
///
/// * `show_name` - Name of the anime show
/// * `episode_number` - Episode number
/// * `mode` - Translation mode (sub/dub)
///
/// # Returns
///
/// A sanitized filename string.
pub fn generate_filename(show_name: &str, episode_number: i64, mode: &str) -> String {
    // Sanitize show name for filesystem
    let safe_name: String = show_name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect();

    format!("{} - Episode {} [{}].mp4", safe_name, episode_number, mode)
}

/// Get the full output path for a download.
///
/// # Arguments
///
/// * `download_dir` - The download directory
/// * `show_name` - Name of the anime show
/// * `episode_number` - Episode number
/// * `mode` - Translation mode (sub/dub)
///
/// # Returns
///
/// The full path where the file should be saved.
pub fn get_output_path(
    download_dir: &Path,
    show_name: &str,
    episode_number: i64,
    mode: &str,
) -> PathBuf {
    let filename = generate_filename(show_name, episode_number, mode);
    download_dir.join(filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_filename_basic() {
        let filename = generate_filename("My Anime", 1, "sub");
        assert_eq!(filename, "My Anime - Episode 1 [sub].mp4");
    }

    #[test]
    fn test_generate_filename_special_chars() {
        let filename = generate_filename("Test: The Show", 5, "dub");
        assert_eq!(filename, "Test_ The Show - Episode 5 [dub].mp4");
    }

    #[test]
    fn test_generate_filename_all_special() {
        let filename = generate_filename("A/B\\C:D*E?F\"G<H>I|J", 10, "sub");
        assert_eq!(filename, "A_B_C_D_E_F_G_H_I_J - Episode 10 [sub].mp4");
    }

    #[test]
    fn test_get_output_path() {
        let path = get_output_path(Path::new("/downloads"), "Test Show", 3, "sub");
        assert_eq!(
            path,
            PathBuf::from("/downloads/Test Show - Episode 3 [sub].mp4")
        );
    }
}
