use crate::git::diff::DiffItem;
use git2::Repository;

pub struct Stash {
    pub index: usize,
    pub message: String,
    pub id: git2::Oid,
}

impl Stash {
    pub fn diff(&self, repo: &Repository) -> anyhow::Result<Vec<DiffItem>> {
        let commit_id = repo.find_commit(self.id)?.id();
        let binding = repo;

        let stash_commit = binding.find_commit(commit_id)?;
        let stash_tree = binding.find_tree(stash_commit.tree_id())?;

        let diff = binding.diff_tree_to_workdir_with_index(
            Some(&stash_tree),
            Some(git2::DiffOptions::new().reverse(true)),
        )?;

        let mut diffs = Vec::new();
        let mut current_diff: Option<DiffItem> = None;
        let mut current_hunk: Option<crate::git::diff::Hunk> = None;

        diff.print(git2::DiffFormat::Patch, |delta, hunk, line| {
            // File header - start a new diff item
            let old_file = delta
                .old_file()
                .path()
                .map(|p| p.to_string_lossy().to_string());
            let new_file = delta
                .new_file()
                .path()
                .map(|p| p.to_string_lossy().to_string());

            if current_diff.is_none()
                || current_diff.as_ref().unwrap().old_file != old_file
                || current_diff.as_ref().unwrap().new_file != new_file
            {
                if let Some(diff_item) = current_diff.take() {
                    diffs.push(diff_item); // Push the previous diff item
                }

                current_diff = Some(DiffItem {
                    old_file,
                    new_file,
                    hunks: Vec::new(),
                });
            }

            if let Some(hunk) = hunk {
                // Hunk header - start a new hunk
                if current_hunk.is_none()
                    || current_hunk.as_ref().unwrap().old_start != hunk.old_start() as usize
                    || current_hunk.as_ref().unwrap().new_start != hunk.new_start() as usize
                {
                    if let Some(h) = current_hunk.take() {
                        if let Some(ref mut diff_item) = current_diff {
                            diff_item.hunks.push(h); // Push the previous hunk
                        }
                    }

                    current_hunk = Some(crate::git::diff::Hunk {
                        old_start: hunk.old_start() as usize,
                        old_lines: hunk.old_lines() as usize,
                        new_start: hunk.new_start() as usize,
                        new_lines: hunk.new_lines() as usize,
                        lines: Vec::new(),
                    });
                }
            }

            // Line changes
            let line_str = String::from_utf8_lossy(line.content()).to_string();
            let line_change = crate::git::diff::LineChange {
                origin: line.origin(),
                content: line_str,
            };

            match line.origin() {
                '-' | '+' | ' ' => {
                    if let Some(ref mut hunk) = current_hunk {
                        hunk.lines.push(line_change); // Push only real line changes
                    }
                }
                _ => {} // Ignore lines that are not part of the diff (e.g., hunk headers)
            }

            true
        })?;
        // Push any remaining diff and hunk
        if let Some(hunk) = current_hunk {
            if let Some(ref mut diff_item) = current_diff {
                diff_item.hunks.push(hunk);
            }
        }
        if let Some(diff_item) = current_diff {
            diffs.push(diff_item);
        }

        Ok(diffs)
    }
}

fn origin_to_diff_line_type(origin: char) -> git2::DiffLineType {
    match origin {
        ' ' => git2::DiffLineType::Context,
        '+' => git2::DiffLineType::Addition,
        '-' => git2::DiffLineType::Deletion,
        _ => git2::DiffLineType::Context,
    }
}
