mod git;

use clap::{Parser, Subcommand};
use ratatui::{
    backend::Backend,
};
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

            let (branch, orphaned_stashes) = repo.get_stashes()?;
            // print out
            println!("{}", branch);
            println!();
            println!("Orphaned stashes: {:?}", orphaned_stashes);
        }
        None => {
            println!("Default subcommand");
        }
    }

    Ok(())
}
