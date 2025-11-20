//! UI rendering functions for the TUI.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::state::App;
use super::types::{Focus, Screen};

/// Draw the UI.
pub fn draw(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Search bar
            Constraint::Min(0),    // Content (sidebar + main)
            Constraint::Length(3), // Footer
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
            Constraint::Length(30), // Sidebar (fixed width)
            Constraint::Min(0),     // Main content
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

    // Draw batch confirmation popup if in confirmation mode
    if app.batch_confirm_mode {
        draw_batch_confirm_popup(frame, app);
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
        Span::styled(
            "anime-watcher",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(format!("[{}]", app.mode), mode_style),
        Span::raw("  "),
        Span::styled(
            format!("[{}]", app.quality),
            Style::default().fg(Color::Green),
        ),
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
        frame.set_cursor_position((area.x + app.search_input.len() as u16 + 1, area.y + 1));
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
                // Truncate name if too long (use chars to avoid UTF-8 panics)
                let display_name = if name.chars().count() > 20 {
                    format!("{}...", name.chars().take(17).collect::<String>())
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
        - q: Quit",
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search Results"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
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
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, chunks[1], &mut app.episode_list_state);

    // Episode details from filtered list
    let details = if let Some(i) = app.episode_list_state.selected() {
        let filtered = app.get_filtered_episodes();
        if i < filtered.len() {
            let episode = filtered[i];
            let action = if app.download_mode {
                "download"
            } else {
                "stream"
            };
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

fn draw_quality_select(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .sources
        .iter()
        .map(|s| ListItem::new(s.to_display()))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Select Quality"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
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
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Download Mode"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
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

    let popup = Paragraph::new(format!("Enter range (e.g., 1-12): {}", input)).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Episode Range"),
    );

    frame.render_widget(popup, area);

    // Show cursor
    frame.set_cursor_position((area.x + 26 + input.len() as u16, area.y + 1));
}

fn draw_batch_confirm_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 20, frame.area());
    frame.render_widget(Clear, area);

    let count = app.get_pending_batch_count();
    let message = format!(
        "Download {} episode{}?\n\n[Y/Enter] Yes  [N/Esc] No",
        count,
        if count == 1 { "" } else { "s" }
    );

    let popup = Paragraph::new(message)
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Confirm Download")
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(popup, area);
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
            let content = format!("{}{}Press ? to close", global_keys, search_keys);
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
            let content = format!("{}{}Press ? to close", global_keys, navigation_keys);
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
            let content = format!("{}Press ? to close", global_keys);
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
