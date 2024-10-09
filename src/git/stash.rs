use crate::git::diff::DiffItem;

pub struct Stash {
    pub index: usize,
    pub message: String,
    pub id: git2::Oid,
    pub commit_id: git2::Oid,

    pub diffs: Vec<DiffItem>,
}
