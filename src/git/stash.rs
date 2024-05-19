use git2::{Commit, DiffFormat,Repository};
use std::error::Error;
use std::time::SystemTime;

#[derive(Debug)]
pub struct Stash<'repo> {
    pub index: usize,
    pub message: String,
    pub id: git2::Oid,
    pub base_commit: Commit<'repo>,
    pub stash_commit: Commit<'repo>,
    pub timestamp: SystemTime,
}

impl Stash<'_> {
    pub fn print_diff(&self, repo: &Repository) -> Result<(), Box<dyn Error>> {
        print_stash_diff(repo, self)
    }
}

impl std::fmt::Display for Stash<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.base_commit.tree_id(), self.message)
    }
}

pub fn get_stashes(repo: &mut Repository) -> Result<Vec<Stash>, Box<dyn Error>> {
    let mut stashes = Vec::new();

    let stash_ids: Vec<(usize, String, git2::Oid)> = {
        let mut ids = Vec::new();
        repo.stash_foreach(|index, message, &id| {
            ids.push((index, message.to_string(), id));
            true
        })?;
        ids
    };

    for (index, message, id) in stash_ids {
        let stash_commit = repo.find_commit(id).expect("Could not retrieve stash object");
        let base_commit = stash_commit.parent(0).expect("Could not retrieve stash commit");

        let time = stash_commit.time();
        let timestamp = SystemTime::UNIX_EPOCH + std::time::Duration::new(time.seconds() as u64, 0);

        stashes.push(Stash {
            index,
            message,
            id,
            stash_commit,
            base_commit,
            timestamp
        });
    }

    Ok(stashes)
}

fn print_stash_diff(repo: &Repository, stash: &Stash) -> Result<(), Box<dyn Error>> {
    let (stash_commit, base_commit) = (
        &stash.stash_commit,
        &stash.base_commit,
    );

    let diff = repo.diff_tree_to_tree(
        Some(&base_commit.tree()?),
        Some(&stash_commit.tree()?),
        None,
    )?;

    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        print!("{}", String::from_utf8_lossy(line.content()));
        true
    })?;

    Ok(())
}
