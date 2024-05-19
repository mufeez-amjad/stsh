use git2::{DiffFormat, Repository};
use std::error::Error;

pub struct Stash {
    pub index: usize,
    pub message: String,
    pub id: git2::Oid,
}

pub fn get_stashes(repo: &mut Repository) -> Result<Vec<Stash>, Box<dyn Error>> {
    let mut stashes = Vec::new();

    repo.stash_foreach(|index, message, &id| {
        stashes.push(Stash {
            index,
            message: message.to_string(),
            id,
        });
        true
    })?;

    Ok(stashes)
}

fn print_stash_diff(repo: &Repository, stash: &Stash) -> Result<(), Box<dyn Error>> {
    let stash_commit = repo.find_commit(stash.id)?;
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
