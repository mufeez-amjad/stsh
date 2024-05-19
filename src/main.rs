mod git;

use clap::{Parser, Subcommand};
use git2::{BranchType, Commit, Repository};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    terminal::{Frame, Terminal},
    widgets::{Block, Borders, Paragraph},
};
use std::error::Error;

use crossterm::event::{self, Event, KeyCode};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Show,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    git::print_result()?;

    // match &cli.command {
    //     Some(Commands::Show) => show_tree()?,
    //     None => {
    //         println!("Default subcommand");
    //     }
    // }

    Ok(())
}

fn show_tree() -> Result<(), Box<dyn Error>> {
    // Initialize the terminal
    let stdout = io::stdout();
    execute!(io::stdout(), EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut repo = Repository::open(".")?;
    let branches = get_branches(&repo)?;

    let stashes = git::stash::get_stashes(&mut repo)?;

    // Main event loop
    loop {
        terminal.draw(|f| ui(f, &branches, &stashes))?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                break;
            }
        }
    }

    // Cleanup
    execute!(io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn get_branches(repo: &Repository) -> Result<Vec<String>, git2::Error> {
    let mut branches = vec![];
    let branch_iter = repo.branches(Some(BranchType::Local))?;
    for branch in branch_iter {
        let (branch, _) = branch?;
        let name = branch.name()?.unwrap_or("").to_string();
        branches.push(name);
    }
    Ok(branches)
}

fn ui(f: &mut Frame, branches: &[String], stashes: &Vec<git::stash::Stash>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let mut tree_text = String::new();
    for branch in branches {
        tree_text.push_str(&format!("Branch: {}\n", branch));
        // tree_text.push_str(&format_commit_tree(&commit, 0, repo));
        tree_text.push_str("\n");
    }

    for stash in stashes {
        tree_text.push_str(&format!("Stash: {} ({})\n", stash.index, stash.message));
        // if let Ok(commit) = repo.find_commit(stash.id) {
        //     tree_text.push_str(&format_commit_tree(&commit, 0, repo));
        // }
        tree_text.push_str("\n");
    }

    let paragraph = Paragraph::new(tree_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Git Branches and Stashes"),
        )
        .style(Style::default().fg(Color::White).bg(Color::Black));

    f.render_widget(paragraph, chunks[0]);
}

// fn ui(f: &mut Frame, repo: &Repository, branches: &[String], stashes: &Vec<git::stash::Stash>) {
//     let chunks = Layout::default()
//         .direction(Direction::Vertical)
//         .margin(1)
//         .constraints([Constraint::Percentage(100)].as_ref())
//         .split(f.size());
//
//     let mut tree_text = String::new();
//     for branch in branches {
//         tree_text.push_str(&format!("Branch: {}\n", branch));
//         if let Ok(commit) = repo
//             .revparse_single(&format!("refs/heads/{}", branch))
//             .and_then(|obj| obj.peel_to_commit())
//         {
//             tree_text.push_str(&format_commit_tree(&commit, 0, repo));
//         }
//         tree_text.push_str("\n");
//     }
//
//     for stash in stashes {
//         tree_text.push_str(&format!("Stash: {} ({})\n", stash.index, stash.message));
//         if let Ok(commit) = repo.find_commit(stash.id) {
//             tree_text.push_str(&format_commit_tree(&commit, 0, repo));
//         }
//         tree_text.push_str("\n");
//     }
//
//     let paragraph = Paragraph::new(tree_text)
//         .block(
//             Block::default()
//                 .borders(Borders::ALL)
//                 .title("Git Branches and Stashes"),
//         )
//         .style(Style::default().fg(Color::White).bg(Color::Black));
//
//     f.render_widget(paragraph, chunks[0]);
// }

fn format_commit_tree(commit: &Commit, level: usize, repo: &Repository) -> String {
    let mut result = String::new();
    for _ in 0..level {
        result.push_str("  ");
    }
    let refs = format_refs(commit, repo);
    result.push_str(&format!("* {} {}\n", commit.id(), refs));

    for i in 0..commit.parent_count() {
        if let Ok(parent) = commit.parent(i) {
            result.push_str(&format_commit_tree(&parent, level + 1, repo));
        }
    }
    result
}

fn format_refs(commit: &Commit, repo: &Repository) -> String {
    let mut refs = String::new();
    if let Ok(head) = repo.head() {
        if head.target() == Some(commit.id()) {
            refs.push_str("(HEAD");
            if let Some(name) = head.shorthand() {
                refs.push_str(&format!(", {}", name));
            }
            refs.push_str(") ");
        }
    }

    if let Ok(branches) = repo.branches(Some(BranchType::Local)) {
        for branch in branches {
            if let Ok((branch, _)) = branch {
                if let Ok(name) = branch.name() {
                    if let Some(name) = name {
                        if branch.get().target() == Some(commit.id()) {
                            refs.push_str(&format!("({}) ", name));
                        }
                    }
                }
            }
        }
    }

    refs.trim_end().to_string()
}
