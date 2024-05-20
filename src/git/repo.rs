use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::SystemTime;
use crate::git::stash::Stash;

pub(crate) struct Repository (pub Rc<RefCell<git2::Repository>>);

#[derive(Debug)]
pub struct Branch {
    pub name: String,
    pub refs: Vec<GitRef>,
    pub timestamp: SystemTime,
}

impl std::fmt::Display for Branch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "- {}", self.name)?;

        for git_ref in &self.refs {
            match git_ref {
                GitRef::Stash(stash) => {
                    write!(f, "\n  - {}", stash.message)?;
                }
                GitRef::Branch(branch) => {
                    write!(f, "\n  - {}", branch)?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum GitRef {
    Stash(Stash),
    Branch(Box<Branch>),
}

impl Repository {
    pub fn new(path: PathBuf) -> Self {
        Repository(Rc::new(RefCell::new(git2::Repository::open(path).expect("Could not open repository"))))
    }

    fn list_stashes(&self) -> Vec<Stash> {
        let mut stashes = Vec::new();
        self.0.borrow_mut().stash_foreach(|index, _message, id| {
            stashes.push(Stash {
                index,
                message: _message.to_string(),
                id: id.clone(),
            });
            true
        }).expect("Could not get stashes");

        stashes
    }

    pub fn get_stashes(&self) -> Result<(Branch, Vec<Stash>), Box<dyn Error>> {
        let stashes = self.list_stashes();

        let mut orphan_stashes = Vec::new();

        // Group stashes by branch
        let mut branch_map: HashMap<String, Vec<Stash>> = HashMap::new();
        for stash in stashes {
            if let Some(branch_name) = get_branch_name(&self, &stash.base_commit(&self)?)? {
                branch_map.entry(branch_name).or_insert(Vec::new()).push(stash);
            } else {
                orphan_stashes.push(stash);
            }
        }

        // Sort stashes by timestamp
        for stashes in branch_map.values_mut() {
            stashes.sort_by_key(|s| s.timestamp(&self).unwrap_or(SystemTime::UNIX_EPOCH));
        }

        // Construct the data structure
        let main_branch = get_default_branch_name(&self)?;
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
            GitRef::Stash(stash) => stash.timestamp(&self).unwrap_or(SystemTime::UNIX_EPOCH),
            GitRef::Branch(branch) => {
                branch.refs.first().map(|r| match r {
                    GitRef::Stash(stash) => stash.timestamp(&self).unwrap(),
                    GitRef::Branch(branch) => branch.timestamp,
                }).unwrap_or(SystemTime::UNIX_EPOCH)
            }
        });

        Ok((root_branch, orphan_stashes))
    }

}

fn get_default_branch_name(repo: &Repository) -> Result<String, git2::Error> {
    let repo = repo.0.borrow();
    let head = repo.head()?;

    let name = head.shorthand().expect("Could not get default branch name").to_string();
    Ok(name)
}

// Helper function to get branch name from commit_id
fn get_branch_name(repo: &Repository, commit: &crate::git::stash::Commit) -> Result<Option<String>, git2::Error> {
    let repo = repo.0.borrow();

    let commit_id = commit.id;

    let mut branch_iter = repo.branches(Some(git2::BranchType::Local))?;
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