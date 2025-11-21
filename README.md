# anime-watcher

A command-line anime streaming application written in Rust, inspired by [ani-cli](https://github.com/pystardust/ani-cli).

## Features

- Full-screen TUI (Terminal User Interface) built with ratatui
- Search for anime by name
- Browse available episodes with keyboard navigation
- Stream episodes through mpv (or platform-specific players)
- Download episodes for offline viewing
- Quality selection (best, worst, or specific resolution)
- Navigate between episodes without restarting
- Support for both subbed and dubbed content
- Post-playback menu for easy navigation (next, previous, replay, select)
- Automatic retry with exponential backoff for network errors
- Watch history and resume functionality
- Configuration file support

## Requirements

- [Rust](https://rustup.rs/) (for building)
- [mpv](https://mpv.io/) - video player (Linux)
- [yt-dlp](https://github.com/yt-dlp/yt-dlp) - for extracting video URLs from embed pages
- `setsid` - for process isolation (usually pre-installed on Linux)

### Platform-specific players
- **Linux**: mpv
- **macOS**: iina
- **Windows**: mpv.exe

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/anime-watcher
cd anime-watcher

# Build the project
cargo build --release

# Run
cargo run --release
```

## Usage

```bash
# Run with default settings (sub mode, streaming)
cargo run

# Run in dub mode
cargo run -- -m dub

# Stream with specific quality
cargo run -- -q 720

# Download mode - save episodes to current directory
cargo run -- -D

# Download to specific directory with quality
cargo run -- -D -d ~/Downloads/anime -q 1080

# Batch download (select all, range, or single when prompted)
cargo run -- -D

# Show help
cargo run -- --help
```

### Command-line Options

| Option | Description | Default |
|--------|-------------|---------|
| `-m, --mode` | Translation mode: "sub" or "dub" | sub |
| `-D, --download` | Enable download mode instead of streaming | false |
| `-d, --download-dir` | Directory for downloads | . |
| `-q, --quality` | Preferred quality: "best", "worst", or number (e.g., "1080") | best |
| `-p, --player` | Video player to use (overrides config) | platform default |
| `-l, --log` | Log verbosity: 0=error, 1=warn, 2=info, 3=debug, 4=trace | 1 |

### Configuration File

anime-watcher supports a TOML configuration file to save your preferences. The config file is located at:
- **Linux**: `~/.config/anime-watcher/config.toml`
- **macOS**: `~/Library/Application Support/anime-watcher/config.toml`
- **Windows**: `%APPDATA%/anime-watcher/config.toml`

Example config file:

```toml
# Translation mode: "sub" or "dub"
mode = "sub"

# Preferred quality: "best", "worst", or a number like "1080"
quality = "best"

# Download directory
download_dir = "~/Downloads/anime"

# Custom video player (optional, overrides platform default)
# player = "vlc"

# Additional arguments to pass to the video player
# player_args = ["--fullscreen", "--volume=80"]

# Custom keybindings (all optional, shown with defaults)
# [keybindings]
# up = ["k", "Up"]
# down = ["j", "Down"]
# select = ["Enter"]
# back = ["Backspace", "Esc"]
# quit = ["q", "Esc"]
# search = ["s", "/"]
# toggle_focus = ["Tab"]
# help = ["?"]
# filter = ["f"]
# next = ["n"]
# previous = ["p"]
# replay = ["r"]
# episodes = ["e"]
# new_search = ["s", "n"]
```

#### Keybinding Format

Keybindings support the following formats:
- Single characters: `"j"`, `"k"`, `"q"`
- Special keys: `"Enter"`, `"Esc"`, `"Tab"`, `"Backspace"`, `"Space"`
- Arrow keys: `"Up"`, `"Down"`, `"Left"`, `"Right"`
- With Ctrl modifier: `"Ctrl+c"`, `"Ctrl+q"`

Each action can have multiple keybindings (e.g., `up = ["k", "Up"]`).

#### Supported Players

Common video player configurations:

```toml
# mpv (default on Linux)
player = "mpv"
player_args = ["--fs", "--volume=100"]

# VLC
player = "vlc"
player_args = ["--fullscreen"]

# IINA (default on macOS)
player = "iina"
player_args = ["--pip"]

# Celluloid
player = "celluloid"
player_args = []
```

CLI arguments override config file settings.

### Controls

The TUI uses vim-style navigation by default. Most keybindings can be customized in the config file; Ctrl+C and Ctrl+Q always force quit.

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` | Select |
| `s` / `/` | Search |
| `Backspace` | Go back |
| `q` / `Esc` | Quit |
| `Ctrl+C` | Force quit |

**Playback Menu**:
- `n` - Next episode
- `p` - Previous episode
- `r` - Replay
- `e` - Episode selection

## Project Structure

```
src/
├── main.rs      # Application entry point and event loop
├── lib.rs       # Library exports
├── api.rs       # AllAnime API client
├── config.rs    # Configuration file support
├── download.rs  # Download functionality
├── history.rs   # Watch history tracking
├── tui.rs       # Ratatui TUI components
├── types.rs     # Data structures
└── ui.rs        # Legacy UI types
```

## Running Tests

```bash
cargo test
```

## TODO

### High Priority
- [x] Implement quality selection (currently always uses first available source)
- [x] Add download functionality
- [x] Handle network errors more gracefully
- [x] Add retry logic for failed API requests

### Medium Priority
- [x] Add watch history/tracking
- [x] Resume from last watched episode
- [x] Support for multiple video players
- [x] Add configuration file support
- [x] Implement proper logging with verbosity levels

### Low Priority
- [ ] Add MAL/AniList integration
- [ ] Support for manga reading
- [x] Batch downloading
- [x] Custom keybindings
- [ ] Themes/colors for terminal output

### Technical Debt
- [ ] Remove unused `download_dir` and `log` CLI arguments or implement them
- [ ] Add integration tests with mock API responses
- [ ] Improve error messages with more context
- [ ] Add CI/CD pipeline

## License

MIT

## Acknowledgments

- [ani-cli](https://github.com/pystardust/ani-cli) - The original inspiration for this project
- [AllAnime](https://allanime.to) - Content source
