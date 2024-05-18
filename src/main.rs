use clap::{Parser, Subcommand};
use git2::{DiffFormat, Repository};
use std::error::Error;

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
        Some(Commands::Show) => show_stash_entries()?,
        None => {
            println!("Default subcommand");
        }
    }

    Ok(())
}

fn show_stash_entries() -> Result<(), Box<dyn Error>> {
    let mut repo = Repository::open(".")?;
    let mut stashes = Vec::new();

    repo.stash_foreach(|index, message, &id| {
        stashes.push((index, message.to_string(), id));
        true
    })?;

    for (index, message, id) in stashes {
        println!("Stash {}: {}", index, message);
        if let Err(e) = print_stash_diff(&repo, &id) {
            eprintln!("Error printing diff for stash {}: {}", index, e);
        }
    }

    Ok(())
}

fn print_stash_diff(repo: &Repository, stash_id: &git2::Oid) -> Result<(), Box<dyn Error>> {
    let stash_commit = repo.find_commit(*stash_id)?;
    let parent_commit = stash_commit.parent(0)?;

    let diff = repo.diff_tree_to_tree(
        Some(&parent_commit.tree()?),
        Some(&stash_commit.tree()?),
        None,
    )?;

    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        print!("{}", String::from_utf8_lossy(line.content()));
        true
    })?;

    Ok(())
}
