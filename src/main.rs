mod git;

use std::cell::RefCell;
use clap::{Parser, Subcommand};
use git2::Repository;
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
    let _cli = Cli::parse();

    let repo = RefCell::new(Repository::open(".")?);
    git::get_stashes(&repo)?;

    // match &cli.command {
    //     Some(Commands::Show) => {}
    //     None => {
    //         println!("Default subcommand");
    //     }
    // }

    Ok(())
}
