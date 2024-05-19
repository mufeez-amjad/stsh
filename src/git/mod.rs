pub mod stash;
use git2::{Repository, Oid, Commit, StashApplyOptions, StashFlags, BranchType};
use std::collections::HashMap;
use std::error::Error;
use std::time::SystemTime;
use crate::git::stash::Stash;

use std::cell::{Ref, RefCell};
use std::rc::Rc;

#[derive(Debug)]
struct Branch<'repo> {
    name: String,
    refs: Vec<GitRef<'repo>>,
    timestamp: SystemTime,
}

impl std::fmt::Display for Branch<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "branch {}", self.name)?;
        for reference in &self.refs {
            write!(f, "\n  {}", reference)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
enum GitRef<'repo> {
    Stash(Stash<'repo>),
    Branch(Box<Branch<'repo>>),
}

impl std::fmt::Display for GitRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GitRef::Stash(stash) => write!(f, "{}", stash),
            GitRef::Branch(branch) => write!(f, "{}", branch),
        }
    }
}

pub fn get_stashes<'repo>(repo: &'repo RefCell<Repository>) -> Result<(Branch<'repo>, Vec<Stash<'repo>>), Box<dyn Error>> {
    let stashes = stash::get_stashes(&mut repo.borrow_mut())?;
    let mut orphan_stashes = Vec::new();

    // Group stashes by branch
    let mut branch_map: HashMap<String, Vec<stash::Stash>> = HashMap::new();
    for stash in stashes {
        if let Some(branch_name) = get_branch_name(repo.borrow(), &stash.base_commit)? {
            branch_map.entry(branch_name).or_insert(Vec::new()).push(stash);
        } else {
            orphan_stashes.push(stash);
        }
    }

    // Sort stashes by timestamp
    for stashes in branch_map.values_mut() {
        stashes.sort_by_key(|s| s.timestamp);
    }

    // Construct the data structure
    let main_branch = get_default_branch_name(repo.borrow())?;
    let mut root_branch = Branch {
        name: main_branch.to_string(),
        refs: Vec::new(),
        timestamp: SystemTime::UNIX_EPOCH,
    };

    for (branch_name, stashes) in branch_map {
        if branch_name == main_branch {
            root_branch.refs = stashes.into_iter().map(GitRef::Stash).collect();
            continue;
        }

        let mut branch = Branch {
            name: branch_name.clone(),
            refs: Vec::new(),
            timestamp: SystemTime::UNIX_EPOCH,
        };

        for stash in stashes {
            branch.refs.push(GitRef::Stash(stash));
        }
        root_branch.refs.push(GitRef::Branch(Box::new(branch)));
    }

    // Sort root refs by timestamp
    root_branch.refs.sort_by_key(|r| match r {
        GitRef::Stash(stash) => stash.timestamp,
        GitRef::Branch(branch) => {
            branch.refs.first().map(|r| match r {
                GitRef::Stash(stash) => stash.timestamp,
                GitRef::Branch(branch) => branch.timestamp,
            }).unwrap_or(SystemTime::UNIX_EPOCH)
        }
    });

    Ok((root_branch, orphan_stashes))
}

fn get_default_branch_name(repo: Ref<Repository>) -> Result<String, git2::Error> {
    let head = repo.head()?;
    let head_name = head.name().unwrap_or("").to_string();
    let head_name = head_name.trim_start_matches("refs/heads/");
    Ok(head_name.to_string())
}

// Helper function to get branch name from commit_id
fn get_branch_name(repo: Ref<Repository>, commit: &Commit) -> Result<Option<String>, git2::Error> {
    let commit_id = commit.id();

    let mut branch_iter = repo.branches(Some(BranchType::Local))?;
    while let Some(branch) = branch_iter.next() {
        let (branch, _) = branch?;
        let branch_name = branch.name()?.unwrap_or("").to_string();

        // Get the target of the branch reference
        if let Ok(branch_commit) = branch.get().peel_to_commit() {
            let branch_commit_id = branch_commit.id();

            // Check if the target commit is an ancestor of the branch commit
            if branch_commit_id == commit_id || repo.graph_descendant_of(commit_id, branch_commit_id)? {
                return Ok(Some(branch_name));
            }
        }
    }
    Ok(None)
}