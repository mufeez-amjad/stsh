use crate::git::stash::Stash;
use std::fmt::Display;
use std::path::Path;

pub struct Diff {
    items: Vec<DiffItem>,
    stash: Stash,
}

pub struct DiffItem {
    pub old_file: Option<String>, // Path to the old file (for "a/filename")
    pub new_file: Option<String>, // Path to the new file (for "b/filename")
    pub hunks: Vec<Hunk>,         // List of hunks with changes
}

pub struct Hunk {
    pub old_start: usize,       // Start line in the old file
    pub old_lines: usize,       // Number of lines in the old file
    pub new_start: usize,       // Start line in the new file
    pub new_lines: usize,       // Number of lines in the new file
    pub lines: Vec<LineChange>, // List of changes in the hunk
}

pub struct LineChange {
    pub origin: char,    // '-', '+', or ' ' (context)
    pub content: String, // The content of the line
}

impl Display for DiffItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(old_file) = &self.old_file {
            writeln!(f, "--- a/{}", old_file)?;
        }
        if let Some(new_file) = &self.new_file {
            writeln!(f, "+++ b/{}", new_file)?;
        }

        for hunk in &self.hunks {
            write!(f, "{}", hunk)?;
        }

        Ok(())
    }
}

impl Display for Hunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "@@ -{},{} +{},{} @@",
            self.old_start, self.old_lines, self.new_start, self.new_lines
        )?;

        for line in &self.lines {
            write!(f, "{}", line)?;
        }

        Ok(())
    }
}

impl Display for LineChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.origin, self.content)
    }
}
