use crate::git::diff::DiffItem;
use crate::git::stash::Stash;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

pub(crate) struct Repository(pub Rc<RefCell<git2::Repository>>);

impl Repository {
    /// Open a repository at the given path
    pub fn new(path: PathBuf) -> Self {
        Repository(Rc::new(RefCell::new(
            git2::Repository::open(path).expect("Could not open repository"),
        )))
    }

    pub fn stashes(&self) -> anyhow::Result<Vec<Stash>> {
        let mut stashes = Vec::new();
        self.0
            .borrow_mut()
            .stash_foreach(|index: usize, title: &str, id: &git2::Oid| {
                stashes.push(Stash {
                    index,
                    message: title.to_string(),
                    id: *id,
                });
                true
            })
            .map_err(|e| anyhow::anyhow!(e))?;

        Ok(stashes)
    }

    pub fn stash_diff(&self, stash: &Stash) -> anyhow::Result<Vec<DiffItem>> {
        stash.diff(&self.0.borrow())
    }
}
