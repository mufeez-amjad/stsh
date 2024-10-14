use std::fmt::Display;

/// A single diff item, which can contain multiple hunks
pub struct DiffItem {
    /// Path to the old file (for "a/filename")
    pub old_file: Option<String>,
    /// Path to the new file (for "b/filename")
    pub new_file: Option<String>,
    /// List of hunks with changes
    pub hunks: Vec<Hunk>,
}

/// A single hunk in a diff, which can contain multiple line changes
pub struct Hunk {
    /// Start line in the old file
    pub old_start: usize,
    /// Number of lines in the old file
    pub old_lines: usize,
    /// Start line in the new file
    pub new_start: usize,
    /// Number of lines in the new file
    pub new_lines: usize,
    /// List of changes in the hunk
    pub lines: Vec<LineChange>,
}

/// A single line change in a hunk
pub struct LineChange {
    /// '-', '+', or ' ' (context)
    pub origin: char,
    // The content of the line
    pub content: String,
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
        let color_code = match self.origin {
            '-' => "\x1b[31m", // Red for deletions
            '+' => "\x1b[32m", // Green for additions
            ' ' => "\x1b[0m",  // Reset for context lines
            _ => "\x1b[0m",    // Reset for any other cases
        };
        write!(f, "{}{}{}\x1b[0m", color_code, self.origin, self.content)
    }
}
