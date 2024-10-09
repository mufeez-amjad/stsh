mod git;

use clap::{Parser, Subcommand};
use ratatui::backend::Backend;
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

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Show) => {
            let repo = git::repo::Repository::new(std::env::current_dir()?);

            let stashes = repo.stashes()?;
            for stash in stashes {
                println!("{}", stash.message);
                for diff in stash.diffs {
                    println!("{}", diff);
                }
            }
        }
        None => {
            println!("Default subcommand");
        }
    }

    Ok(())
}
