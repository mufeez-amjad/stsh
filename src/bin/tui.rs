use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::text::Text;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::io::{stdout, Write};

struct App {
    stashes: Vec<String>,
    selected_stash: usize,
    diff: String,
}

impl App {
    fn new() -> App {
        App {
            stashes: vec![
                "stash@{0}: WIP on main: 1a2b3c4 Last commit message".to_string(),
                "stash@{1}: WIP on feature: 5d6e7f8 Another commit message".to_string(),
            ],
            selected_stash: 0,
            diff: "+ Added line\n- Removed line\n  Unchanged line".to_string(),
        }
    }

    fn next_stash(&mut self) {
        if self.selected_stash < self.stashes.len() - 1 {
            self.selected_stash += 1;
        }
    }

    fn previous_stash(&mut self) {
        if self.selected_stash > 0 {
            self.selected_stash -= 1;
        }
    }
}

fn main() -> Result<(), io::Error> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => {
                    // Exit on 'q' press
                    break;
                }
                KeyCode::Down => {
                    // Move down in the stash list
                    app.next_stash();
                }
                KeyCode::Up => {
                    // Move up in the stash list
                    app.previous_stash();
                }
                _ => {}
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
        .split(f.size());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(chunks[0]);

    render_stash_list(f, app, main_chunks[0]);
    render_diff(f, app, main_chunks[1]);
}

fn render_stash_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .stashes
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if i == app.selected_stash {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(s.as_str()).style(style)
        })
        .collect();

    let stashes = List::new(items)
        .block(Block::default().title("Stashes").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(stashes, area);
}

fn render_diff(f: &mut Frame, app: &App, area: Rect) {
    let diff = Paragraph::new(String::from(&app.diff))
        .block(Block::default().title("Diff").borders(Borders::ALL));

    f.render_widget(diff, area);
}
