mod git;
mod tui;

use clap::{Parser, Subcommand};
use ratatui::backend::Backend;
use ratatui::prelude::Widget;
use std::error::Error;
use std::io::Write;

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

#[derive(Debug)]
struct App {
    should_quit: bool,
}

impl App {
    fn ui(&self, f: &mut ratatui::Frame) {
        ratatui::widgets::Block::default()
            .title("Block")
            .borders(ratatui::widgets::Borders::ALL)
            .render(f.size(), f.buffer_mut());

        ratatui::widgets::Paragraph::new("Paragraph")
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Red))
            .block(
                ratatui::widgets::Block::default()
                    .title("Paragraph")
                    .borders(ratatui::widgets::Borders::ALL),
            )
            .wrap(ratatui::widgets::Wrap { trim: true })
            .render(f.size(), f.buffer_mut());

        ratatui::widgets::List::new(vec!["Item1", "Item2", "Item3"])
            .block(
                ratatui::widgets::Block::default()
                    .title("List")
                    .borders(ratatui::widgets::Borders::ALL),
            )
            .render(f.size(), f.buffer_mut());
    }

    #[tui::tracing::instrument]
    fn handle_event(&mut self, evt: tui::Event) -> Option<String> {
        match evt {
            tui::Event::Key(key) => {
                tui::tracing::info!("Key event: {:?}", key);
                None
            }
            tui::Event::Quit => "quit".to_owned().into(),
            _ => None,
        }
    }

    fn update(&mut self, action: String) -> Option<String> {
        match action.as_str() {
            "quit" => {
                self.should_quit = true;
                None
            }
            _ => None,
        }
    }

    async fn run(&mut self) -> anyhow::Result<()> {
        let mut tui = tui::Tui::new()
            .map_err(|e| anyhow::anyhow!(e))?
            .tick_rate(4.0) // 4 ticks per second
            .frame_rate(30.0); // 30 frames per second

        tui.enter().map_err(|e| anyhow::anyhow!(e))?; // Starts event handler, enters raw mode, enters alternate screen

        loop {
            tui.draw(|f| {
                // Deref allows calling `tui.terminal.draw`
                self.ui(f);
            })?;

            if let Some(evt) = tui.next().await {
                // `tui.next().await` blocks till next event
                let mut maybe_action = self.handle_event(evt);
                while let Some(action) = maybe_action {
                    maybe_action = self.update(action);
                }
            };

            if self.should_quit {
                break;
            }
        }

        tui::tracing::info!("Exiting");
        tui.exit().map_err(|e| anyhow::anyhow!(e))?; // Exits alternate screen, exits raw mode

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let mut app = App { should_quit: false };

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.block_on(app.run())?;

    // match &cli.command {
    //     Some(Commands::Show) => {
    //         let repo = git::repo::Repository::new(std::env::current_dir()?);
    //
    //         let stashes = repo.stashes()?;
    //         for stash in stashes {
    //             println!("{}", stash.message);
    //             for diff in stash.diffs {
    //                 println!("{}", diff);
    //             }
    //         }
    //     }
    //     None => {
    //         println!("Default subcommand");
    //     }
    // }

    Ok(())
}
