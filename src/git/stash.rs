use git2::{DiffFormat};
use std::error::Error;
use std::time::SystemTime;

use crate::git::repo::Repository;

#[derive(Debug)]
pub struct Stash {
    pub index: usize,
    pub message: String,
    pub id: git2::Oid,
}

pub struct Commit {
    pub id: git2::Oid,
}

impl Stash {
    pub fn base_commit(&self, repo: &Repository) -> Result<Commit, Box<dyn Error>> {
        let repo = repo.0.borrow();

        let stash_commit = repo.find_commit(self.id)?;
        let base_commit = stash_commit.parent(0)?;

        Ok(Commit {
            id: base_commit.id(),
        })
    }

    pub fn timestamp(&self, repo: &Repository) -> Result<SystemTime, Box<dyn Error>> {
        let repo = repo.0.borrow();

        let stash_commit = repo.find_commit(self.id)?;
        let timestamp = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(stash_commit.time().seconds() as u64);

        Ok(timestamp)
    }

    #[allow(dead_code)]
    pub fn print_diff(&self, repo: &Repository) -> Result<(), Box<dyn Error>> {
        let repo = repo.0.borrow();

        let stash_commit = repo.find_commit(self.id)?;
        let base_commit = stash_commit.parent(0)?;

        let (stash_commit, base_commit) = (
            &stash_commit,
            &base_commit,
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
}
