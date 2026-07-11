use std::{env, error::Error, io, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Debug, Default)]
struct App {
    input: String,
    search_history: Vec<String>,
    search_results: Vec<SearchResult>,
    status_message: String,
}

#[derive(Debug, Clone)]
struct SearchResult {
    url: String,
    title: String,
    score: f32,
}

#[derive(Debug, Deserialize)]
struct ApiSearchResult {
    url: String,
    title: String,
    score: f32,
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App {
        input: String::new(),
        search_history: Vec::new(),
        search_results: Vec::new(),
        status_message: "Ready to search".to_string(),
    };

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {err:?}");
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Enter => {
                        let query = app.input.trim();
                        if !query.is_empty() {
                            app.search_history.push(query.to_string());
                            app.status_message = "Searching...".to_string();
                            app.search_results.clear();
                            match search_api(query) {
                                Ok(results) => {
                                    app.search_results = results;
                                    app.status_message = if app.search_results.is_empty() {
                                        "No matches found".to_string()
                                    } else {
                                        format!("{} result{}", app.search_results.len(), if app.search_results.len() == 1 { "" } else { "s" })
                                    };
                                }
                                Err(err) => {
                                    app.status_message = format!("Search failed: {err}");
                                }
                            }
                            app.input.clear();
                        }
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    _ => {}
                }
            }
        }
    }
}

fn search_api(query: &str) -> Result<Vec<SearchResult>, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|err| err.to_string())?;

    let endpoint = env::var("IXEO_SEARCH_API")
        .unwrap_or_else(|_| "https://ixeo.midelight.net/api/search".to_string());
    let full_url = format!("{endpoint}?q={query}");
    let response = client
        .get(full_url)
        .send()
        .map_err(|err| format!("request failed: {err}"))?;

    if !response.status().is_success() {
        return Err(format!("server returned {}", response.status()));
    }

    let body = response
        .text()
        .map_err(|err| format!("could not read response: {err}"))?;

    let items: Vec<ApiSearchResult> = serde_json::from_str(&body).map_err(|err| {
        format!("invalid JSON from {}: {err}\nbody: {}", endpoint, body.chars().take(300).collect::<String>())
    })?;

    Ok(items
        .into_iter()
        .map(|item| SearchResult {
            url: item.url,
            title: item.title,
            score: item.score,
        })
        .collect())
}

fn gradient_spans(text: &str) -> Vec<Span<'_>> {
    let start = (101, 186, 255);
    let end = (208, 187, 255);
    let chars: Vec<char> = text.chars().collect();

    chars
        .iter()
        .enumerate()
        .map(|(index, ch)| {
            let ratio = if chars.len() <= 1 {
                0.0
            } else {
                index as f32 / (chars.len() - 1) as f32
            };
            let r = (start.0 as f32 + (end.0 as f32 - start.0 as f32) * ratio) as u8;
            let g = (start.1 as f32 + (end.1 as f32 - start.1 as f32) * ratio) as u8;
            let b = (start.2 as f32 + (end.2 as f32 - start.2 as f32) * ratio) as u8;
            Span::styled(
                ch.to_string(),
                Style::default().fg(Color::Rgb(r, g, b)).add_modifier(Modifier::BOLD),
            )
        })
        .collect()
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(f.area());

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(7),
            Constraint::Min(0),
        ])
        .split(chunks[0]);

    let mut main_text = Vec::new();
    main_text.push(Line::from(gradient_spans("IXEO Search")));
    main_text.push(Line::from("Tips for getting started:"));
    main_text.push(Line::from(" 1. Type queries to scan indexes instantly."));
    main_text.push(Line::from(" 2. Press Enter to run the live search API."));
    main_text.push(Line::from(""));

    if let Some(last_search) = app.search_history.last() {
        main_text.push(Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::DarkGray)),
            Span::styled(last_search, Style::default().fg(Color::DarkGray)),
        ]));
    }

    main_text.push(Line::from(vec![
        Span::styled("✦ ", Style::default().fg(Color::Magenta)),
        Span::styled(&app.status_message, Style::default().fg(Color::White)),
    ]));

    f.render_widget(Paragraph::new(main_text), main_layout[0]);

    let result_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let mut result_lines = Vec::new();
    if app.search_results.is_empty() {
        result_lines.push(Line::from("No search results yet. Try a query."));
    } else {
        for (index, result) in app.search_results.iter().take(5).enumerate() {
            result_lines.push(Line::from(vec![
                Span::styled(format!(" {} ", index + 1), Style::default().fg(Color::DarkGray)),
                Span::styled(&result.title, Style::default().fg(Color::Cyan)),
            ]));
            result_lines.push(Line::from(vec![
                Span::styled("   ", Style::default().fg(Color::DarkGray)),
                Span::styled(&result.url, Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    let result_content = Paragraph::new(result_lines).block(result_block);
    f.render_widget(result_content, main_layout[1]);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(101, 186, 255)));

    let input_paragraph = if app.input.is_empty() {
        Paragraph::new(Line::from(vec![
            Span::styled(" > ", Style::default().fg(Color::Rgb(101, 186, 255))),
            Span::styled("Type your query or data path...", Style::default().fg(Color::DarkGray)),
        ]))
    } else {
        Paragraph::new(Line::from(vec![
            Span::styled(" > ", Style::default().fg(Color::Rgb(101, 186, 255))),
            Span::styled(&app.input, Style::default().fg(Color::White)),
        ]))
    };

    f.render_widget(input_paragraph.block(input_block), chunks[1]);

    let status_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    let left_status = Paragraph::new(Line::from(vec![
        Span::styled(" ~/src/client/engine", Style::default().fg(Color::Cyan)),
        Span::styled("  api: https://ixeo.midelight.net/api/search ", Style::default().fg(Color::LightRed)),
    ]))
    .alignment(Alignment::Left);

    let right_status = Paragraph::new(Line::from(vec![
        Span::styled("gradient-blue-purple ", Style::default().fg(Color::DarkGray)),
        Span::styled("live ", Style::default().fg(Color::Green)),
    ]))
    .alignment(Alignment::Right);

    f.render_widget(left_status, status_chunks[0]);
    f.render_widget(right_status, status_chunks[1]);
}

#[cfg(test)]
mod tests {
    use super::gradient_spans;

    #[test]
    fn gradient_spans_keep_text_length() {
        let spans = gradient_spans("IXEO Search");
        assert_eq!(spans.len(), "IXEO Search".chars().count());
    }
}