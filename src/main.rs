//! Main entry point for the anime-watcher CLI application.

mod api;
mod config;
mod download;
mod error;
mod history;
mod tui;
mod types;

use crate::api::{fetch_episodes, fetch_stream_sources, search_shows};
use crate::config::Config;
use crate::download::{download_file, get_output_path};
use crate::history::WatchHistory;
use crate::tui::{draw, poll_event, Action, App};
use crate::types::StreamSource;
use clap::Parser;
use crossterm::{
    event::Event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::{debug, info, warn};
use ratatui::prelude::*;
use std::env;
use std::io::{self, stdout};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

/// Command-line arguments for the anime-watcher application.
#[derive(Parser, Debug)]
#[command(
    name = "anime-watcher",
    version,
    about = "A CLI anime streaming application",
    long_about = "Search, browse, and stream anime from AllAnime using a TUI interface."
)]
struct Args {
    /// Translation mode: "sub" for subtitled, "dub" for dubbed
    #[arg(short, long, default_value = "sub")]
    mode: String,

    /// Directory for downloads
    #[arg(short, long, default_value = ".")]
    download_dir: String,

    /// Enable download mode instead of streaming
    #[arg(short = 'D', long)]
    download: bool,

    /// Log verbosity level: 0=error, 1=warn, 2=info, 3=debug, 4=trace
    #[arg(short, long, default_value_t = 1)]
    log: u8,

    /// Preferred video quality: "best", "worst", or a number (e.g., "1080", "720")
    #[arg(short, long, default_value = "best")]
    quality: String,

    /// Video player to use (overrides config and platform default)
    #[arg(short, long)]
    player: Option<String>,
}

/// Search for an executable in the system PATH.
///
/// Handles:
/// - Absolute paths (checked directly)
/// - Relative paths with separators (checked directly)
/// - Windows PATHEXT extensions (.exe, .cmd, .bat)
/// - Standard PATH search
fn find_in_path<P: AsRef<Path>>(exe_name: P) -> Option<PathBuf> {
    let exe_path = exe_name.as_ref();

    // If it's an absolute path or contains path separators, check it directly
    if exe_path.is_absolute()
        || exe_path
            .to_string_lossy()
            .contains(std::path::MAIN_SEPARATOR)
    {
        if exe_path.is_file() {
            return Some(exe_path.to_path_buf());
        }
        // On Windows, also try with common extensions
        #[cfg(windows)]
        {
            for ext in &[".exe", ".cmd", ".bat", ".com"] {
                let with_ext = exe_path.with_extension(&ext[1..]);
                if with_ext.is_file() {
                    return Some(with_ext);
                }
            }
        }
        return None;
    }

    env::var_os("PATH").and_then(|paths| {
        // Get PATHEXT on Windows for executable extensions
        #[cfg(windows)]
        let extensions: Vec<String> = env::var("PATHEXT")
            .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string())
            .split(';')
            .map(|s| s.to_lowercase())
            .collect();

        env::split_paths(&paths).find_map(|dir| {
            let full_path = dir.join(&exe_name);

            // Check exact name first
            if full_path.is_file() {
                return Some(full_path);
            }

            // On Windows, try with PATHEXT extensions
            #[cfg(windows)]
            {
                for ext in &extensions {
                    let ext_trimmed = ext.trim_start_matches('.');
                    let with_ext = full_path.with_extension(ext_trimmed);
                    if with_ext.is_file() {
                        return Some(with_ext);
                    }
                }
            }

            None
        })
    })
}

/// Select a stream source based on quality preference.
fn choose_stream(
    sources: &[StreamSource],
    quality: &str,
) -> Result<StreamSource, Box<dyn std::error::Error>> {
    if sources.is_empty() {
        return Err("No sources available".into());
    }

    if sources.len() == 1 {
        return Ok(sources[0].clone());
    }

    let mut known_quality: Vec<&StreamSource> = sources.iter().filter(|s| s.quality > 0).collect();
    let unknown_quality: Vec<&StreamSource> = sources.iter().filter(|s| s.quality == 0).collect();

    known_quality.sort_by(|a, b| b.quality.cmp(&a.quality));

    match quality.to_lowercase().as_str() {
        "best" => {
            if let Some(source) = known_quality.first() {
                Ok((*source).clone())
            } else if let Some(source) = unknown_quality.first() {
                Ok((*source).clone())
            } else {
                Ok(sources[0].clone())
            }
        }
        "worst" => {
            if let Some(source) = known_quality.last() {
                Ok((*source).clone())
            } else if let Some(source) = unknown_quality.first() {
                Ok((*source).clone())
            } else {
                Ok(sources[0].clone())
            }
        }
        q => {
            if let Ok(target_quality) = q.parse::<i32>() {
                if let Some(source) = known_quality.iter().find(|s| s.quality == target_quality) {
                    return Ok((*source).clone());
                }

                if !known_quality.is_empty() {
                    let closest = known_quality
                        .iter()
                        .min_by_key(|s| (s.quality - target_quality).abs())
                        .unwrap();
                    return Ok((*closest).clone());
                }

                Ok(sources[0].clone())
            } else {
                // Return first source if quality string is invalid
                Ok(sources[0].clone())
            }
        }
    }
}

/// Get the appropriate video player for the current operating system.
fn get_player() -> Result<&'static str, String> {
    match std::env::consts::OS {
        "linux" => Ok("mpv"),
        "windows" => Ok("mpv.exe"),
        "macos" => Ok("iina"),
        other => Err(format!("OS '{}' is not supported", other)),
    }
}

/// Initialize the terminal for TUI rendering.
fn init_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    Terminal::new(backend)
}

/// Restore the terminal to its original state.
fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize logging
    let log_level = match args.log {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Warn,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    env_logger::Builder::new()
        .filter_level(log_level)
        .format_timestamp(None)
        .format_target(false)
        .init();

    debug!("Log level set to {:?}", log_level);

    // Load config
    let config = Config::load().unwrap_or_else(|e| {
        warn!("Failed to load config: {}. Using defaults.", e);
        Config::new()
    });

    // Merge config with CLI args
    let mode_str = if args.mode == "sub" {
        config.mode.clone()
    } else {
        args.mode.clone()
    };

    let quality_str = if args.quality == "best" {
        config.quality.clone()
    } else {
        args.quality.clone()
    };

    let download_dir_str = if args.download_dir == "." {
        config.download_dir.clone()
    } else {
        args.download_dir.clone()
    };

    // Validate mode
    let mode = match mode_str.as_str() {
        "sub" | "dub" => mode_str.clone(),
        other => {
            eprintln!("Error: Invalid mode '{}'. Use 'sub' or 'dub'.", other);
            std::process::exit(1);
        }
    };

    let download_dir = Path::new(&download_dir_str);
    let download_mode = args.download;
    let quality = quality_str.clone();

    // Verify download directory
    if download_mode && !download_dir.exists() {
        eprintln!(
            "Error: Download directory '{}' does not exist.",
            download_dir.display()
        );
        std::process::exit(1);
    }

    // Verify yt-dlp is available (required for stream extraction)
    if find_in_path("yt-dlp").is_none() {
        eprintln!("Error: yt-dlp not found in PATH. Please install yt-dlp.");
        eprintln!("       Visit: https://github.com/yt-dlp/yt-dlp#installation");
        std::process::exit(1);
    }

    // Get player
    let player: String = if let Some(cli_player) = &args.player {
        cli_player.clone()
    } else if let Some(config_player) = &config.player {
        config_player.clone()
    } else {
        match get_player() {
            Ok(p) => p.to_string(),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    };

    let player_args = config.player_args.clone();

    if find_in_path(&player).is_none() {
        eprintln!("Error: {} not found in PATH.", player);
        std::process::exit(1);
    }

    info!("Using video player: {}", player);

    // Load watch history
    let mut watch_history = WatchHistory::load().unwrap_or_default();

    // Initialize terminal
    let mut terminal = init_terminal()?;

    // Create app state
    let mut app = App::new(mode.clone(), quality.clone(), download_mode);

    // Set up history for startup screen
    let recent = watch_history.get_recent(10);
    let history_records: Vec<(String, String, i64, String)> = recent
        .iter()
        .map(|r| {
            (
                r.show_id.clone(),
                r.show_name.clone(),
                r.episode,
                r.mode.clone(),
            )
        })
        .collect();
    app.set_history(history_records);

    // Main event loop
    let result = run_app(
        &mut terminal,
        &mut app,
        &mut watch_history,
        &mode,
        &quality,
        download_dir,
        &player,
        &player_args,
    )
    .await;

    // Restore terminal
    restore_terminal()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    watch_history: &mut WatchHistory,
    mode: &str,
    quality: &str,
    download_dir: &Path,
    player: &str,
    player_args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // Draw UI
        terminal.draw(|f| draw(f, app))?;

        // Poll for events
        if let Some(event) = poll_event(Duration::from_millis(100))? {
            if let Event::Key(key) = event {
                let action = app.handle_input(key);

                match action {
                    Action::Quit => break,
                    Action::Search(ref query) => {
                        app.set_loading(&format!("Searching for '{}'...", query));
                        terminal.draw(|f| draw(f, app))?;

                        match search_shows(query, mode).await {
                            Ok(shows) => {
                                if shows.is_empty() {
                                    app.set_error("No results found");
                                    app.screen = tui::Screen::Search;
                                } else {
                                    app.set_shows(shows);
                                }
                            }
                            Err(e) => {
                                app.set_error(&e.to_string());
                                app.screen = tui::Screen::Search;
                            }
                        }
                    }
                    Action::SelectShow(i) => {
                        if i < app.shows.len() {
                            let show = app.shows[i].clone();
                            app.selected_show = Some(show.clone());
                            app.set_loading(&format!("Loading episodes for {}...", show.name));
                            terminal.draw(|f| draw(f, app))?;

                            match fetch_episodes(&show.id, mode).await {
                                Ok(mut episodes) => {
                                    episodes.sort_by_key(|e| e.number);
                                    app.set_episodes(episodes);
                                }
                                Err(e) => {
                                    app.set_error(&e.to_string());
                                    app.screen = tui::Screen::ShowList;
                                }
                            }
                        }
                    }
                    Action::SelectEpisode(i) => {
                        if i < app.episodes.len() {
                            let episode = app.episodes[i].clone();
                            app.current_episode = Some(episode.clone());

                            if app.download_mode {
                                app.show_batch_menu();
                            } else {
                                // Fetch sources and play
                                if let Some(show) = app.selected_show.clone() {
                                    app.set_loading("Fetching stream sources...");
                                    terminal.draw(|f| draw(f, app))?;

                                    let episode_str = episode.number.to_string();
                                    match fetch_stream_sources(&show.id, mode, &episode_str).await {
                                        Ok(sources) => {
                                            if sources.is_empty() {
                                                app.set_error("No sources found");
                                                app.screen = tui::Screen::EpisodeList;
                                            } else {
                                                // Auto-select quality and play
                                                match choose_stream(&sources, quality) {
                                                    Ok(source) => {
                                                        app.selected_source = Some(source.clone());

                                                        // Save history
                                                        watch_history.update(
                                                            &show.id,
                                                            &show.name,
                                                            episode.number,
                                                            mode,
                                                        );
                                                        let _ = watch_history.save();

                                                        // Spawn player
                                                        let mut cmd = Command::new("setsid");
                                                        cmd.arg(player);
                                                        for arg in player_args {
                                                            cmd.arg(arg);
                                                        }
                                                        cmd.arg(&source.url);

                                                        debug!("Playing: {}", source.url);

                                                        let _ = cmd
                                                            .stdin(Stdio::null())
                                                            .stdout(Stdio::null())
                                                            .stderr(Stdio::null())
                                                            .spawn();

                                                        app.show_playback_menu();
                                                    }
                                                    Err(e) => {
                                                        app.set_error(&e.to_string());
                                                        app.screen = tui::Screen::EpisodeList;
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            app.set_error(&e.to_string());
                                            app.screen = tui::Screen::EpisodeList;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Action::SelectQuality(i) => {
                        if i < app.sources.len() {
                            app.selected_source = Some(app.sources[i].clone());
                            // This would be used for manual quality selection
                        }
                    }
                    Action::Next | Action::Previous | Action::Replay => {
                        if let Some(current_ep) = &app.current_episode {
                            let current_idx = app
                                .episodes
                                .iter()
                                .position(|e| e.number == current_ep.number);

                            let new_episode = match action {
                                Action::Next => {
                                    current_idx.and_then(|i| app.episodes.get(i + 1).cloned())
                                }
                                Action::Previous => current_idx.and_then(|i| {
                                    if i > 0 {
                                        app.episodes.get(i - 1).cloned()
                                    } else {
                                        None
                                    }
                                }),
                                Action::Replay => Some(current_ep.clone()),
                                _ => None,
                            };

                            if let Some(episode) = new_episode {
                                app.current_episode = Some(episode.clone());

                                if let Some(show) = app.selected_show.clone() {
                                    app.set_loading("Fetching stream sources...");
                                    terminal.draw(|f| draw(f, app))?;

                                    let episode_str = episode.number.to_string();
                                    match fetch_stream_sources(&show.id, mode, &episode_str).await {
                                        Ok(sources) => {
                                            if let Ok(source) = choose_stream(&sources, quality) {
                                                // Save history
                                                watch_history.update(
                                                    &show.id,
                                                    &show.name,
                                                    episode.number,
                                                    mode,
                                                );
                                                let _ = watch_history.save();

                                                // Spawn player
                                                let mut cmd = Command::new("setsid");
                                                cmd.arg(player);
                                                for arg in player_args {
                                                    cmd.arg(arg);
                                                }
                                                cmd.arg(&source.url);

                                                let _ = cmd
                                                    .stdin(Stdio::null())
                                                    .stdout(Stdio::null())
                                                    .stderr(Stdio::null())
                                                    .spawn();

                                                app.show_playback_menu();
                                            }
                                        }
                                        Err(e) => {
                                            app.set_error(&e.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Action::BackToEpisodes => {
                        app.screen = tui::Screen::EpisodeList;
                    }
                    Action::ContinueFromHistory(i) => {
                        if i < app.history_records.len() {
                            let (show_id, show_name, episode_num, record_mode) =
                                app.history_records[i].clone();

                            app.set_loading(&format!("Loading {}...", show_name));
                            terminal.draw(|f| draw(f, app))?;

                            match fetch_episodes(&show_id, &record_mode).await {
                                Ok(mut episodes) => {
                                    episodes.sort_by_key(|e| e.number);

                                    // Find episode to resume
                                    let resume_ep = episodes
                                        .iter()
                                        .find(|e| e.number == episode_num + 1)
                                        .or_else(|| {
                                            episodes.iter().find(|e| e.number == episode_num)
                                        })
                                        .cloned()
                                        .unwrap_or_else(|| episodes[0].clone());

                                    app.selected_show = Some(types::Show {
                                        id: show_id,
                                        name: show_name,
                                        available_episodes: episodes.len() as i64,
                                    });
                                    app.set_episodes(episodes);

                                    // Select the resume episode
                                    let idx = app
                                        .episodes
                                        .iter()
                                        .position(|e| e.number == resume_ep.number)
                                        .unwrap_or(0);
                                    app.episode_list_state.select(Some(idx));
                                }
                                Err(e) => {
                                    app.set_error(&e.to_string());
                                    app.screen = tui::Screen::Startup;
                                }
                            }
                        }
                    }
                    Action::NewSearch => {
                        app.screen = tui::Screen::Search;
                    }
                    Action::BatchAll | Action::BatchSingle | Action::BatchRange(_, _) => {
                        let show = app.selected_show.clone();
                        let current_ep = app.current_episode.clone();
                        if let (Some(show), Some(_)) = (show, current_ep) {
                            let episodes_to_download: Vec<_> = match &action {
                                Action::BatchAll => app.episodes.clone(),
                                Action::BatchRange(start, end) => app
                                    .episodes
                                    .iter()
                                    .filter(|e| e.number >= *start && e.number <= *end)
                                    .cloned()
                                    .collect(),
                                Action::BatchSingle => {
                                    vec![app.current_episode.clone().unwrap()]
                                }
                                _ => vec![],
                            };

                            // Perform batch download
                            let total = episodes_to_download.len();
                            for (idx, episode) in episodes_to_download.iter().enumerate() {
                                let output_path =
                                    get_output_path(download_dir, &show.name, episode.number, mode);

                                if output_path.exists() {
                                    app.set_status(&format!(
                                        "[{}/{}] Skipping {} (exists)",
                                        idx + 1,
                                        total,
                                        output_path.display()
                                    ));
                                    terminal.draw(|f| draw(f, app))?;
                                    continue;
                                }

                                app.set_loading(&format!(
                                    "[{}/{}] Downloading Episode {}...",
                                    idx + 1,
                                    total,
                                    episode.number
                                ));
                                terminal.draw(|f| draw(f, app))?;

                                let episode_str = episode.number.to_string();
                                match fetch_stream_sources(&show.id, mode, &episode_str).await {
                                    Ok(sources) if !sources.is_empty() => {
                                        if let Ok(source) = choose_stream(&sources, quality) {
                                            match download_file(&source.url, &output_path).await {
                                                Ok(()) => {
                                                    watch_history.update(
                                                        &show.id,
                                                        &show.name,
                                                        episode.number,
                                                        mode,
                                                    );
                                                    let _ = watch_history.save();
                                                }
                                                Err(e) => {
                                                    app.set_error(&format!(
                                                        "Download failed: {}",
                                                        e
                                                    ));
                                                    terminal.draw(|f| draw(f, app))?;
                                                    tokio::time::sleep(Duration::from_secs(1))
                                                        .await;
                                                    app.clear_error();
                                                }
                                            }
                                        }
                                    }
                                    _ => {
                                        app.set_error(&format!(
                                            "No sources for episode {}",
                                            episode.number
                                        ));
                                        terminal.draw(|f| draw(f, app))?;
                                        tokio::time::sleep(Duration::from_secs(1)).await;
                                        app.clear_error();
                                    }
                                }
                            }

                            app.set_status("Download complete!");
                            app.screen = tui::Screen::EpisodeList;
                        }
                    }
                    Action::Stream | Action::Download | Action::None => {}
                }

                // Clear error after any input
                if !matches!(action, Action::None) {
                    app.clear_error();
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
