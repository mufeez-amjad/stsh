pub mod stash;
pub mod tree;
use git2::{Repository, Oid, Commit, StashApplyOptions, StashFlags, BranchType};
use std::collections::HashMap;
use std::error::Error;
use std::time::SystemTime;

// Define the custom structs
#[derive(Debug)]
struct Branch<'repo> {
    name: String,
    refs: Vec<Ref<'repo>>,
    // base_commit: Commit<'repo>,
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
enum Ref<'repo> {
    Stash(stash::Stash<'repo>),
    Branch(Box<Branch<'repo>>),
}

impl std::fmt::Display for Ref<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Ref::Stash(stash) => write!(f, "{}", stash),
            Ref::Branch(branch) => write!(f, "{}", branch),
        }
    }
}

pub fn print_result() -> Result<(), Box<dyn Error>> {
    let mut repo = Repository::open(".")?;

    let stashes = stash::get_stashes(&mut repo)?;
    let mut orphan_stashes = Vec::new();

    let repo = Repository::open(".")?;

    // Group stashes by branch
    let mut branch_map: HashMap<String, Vec<stash::Stash>> = HashMap::new();
    for stash in stashes {
        if let Some(branch_name) = get_branch_name(&repo, &stash.base_commit)? {
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
    let main_branch = "main"; // TODO: Get the main branch from the repo
    let mut root_branch = Branch {
        name: main_branch.to_string(),
        refs: Vec::new(),
        // base_commit: repo.find_branch(main_branch, git2::BranchType::Local)?.get().peel_to_commit()?,
        timestamp: SystemTime::UNIX_EPOCH,
    };

    for (branch_name, stashes) in branch_map {
        if (branch_name == main_branch) {
            root_branch.refs = stashes.into_iter().map(Ref::Stash).collect();
            continue;
        }

        let mut branch = Branch {
            name: branch_name.clone(),
            refs: Vec::new(),
            // base_commit: repo.find_branch(&branch_name, git2::BranchType::Local)?.get().peel_to_commit()?,
            timestamp: SystemTime::UNIX_EPOCH,
        };

        for stash in stashes {
            branch.refs.push(Ref::Stash(stash));
        }
        root_branch.refs.push(Ref::Branch(Box::new(branch)));
    }

    // Sort root refs by timestamp
    root_branch.refs.sort_by_key(|r| match r {
        Ref::Stash(stash) => stash.timestamp,
        Ref::Branch(branch) => {
            branch.refs.first().map(|r| match r {
                Ref::Stash(stash) => stash.timestamp,
                Ref::Branch(branch) => branch.timestamp,
            }).unwrap_or(SystemTime::UNIX_EPOCH)
        }
    });

    // Print the result for debugging purposes
    println!("{}", root_branch);

    Ok(())
}

// Helper function to get branch name from commit_id
fn get_branch_name<'repo>(repo: &'repo Repository, commit: &Commit<'repo>) -> Result<Option<String>, git2::Error> {
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