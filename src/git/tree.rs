use git2::{BranchType, Commit, Oid, Repository, Tree};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    terminal::Frame,
    widgets::{Block, Borders, Paragraph},
};

pub struct Branch<'repo> {
    pub name: String,
    pub target: Commit<'repo>,
}

pub fn get_head_tree<'repo>(repo: &'repo Repository) -> Result<Tree<'repo>, git2::Error> {
    let head = repo.head()?;
    let tree = head.peel_to_tree()?;
    Ok(tree)
}

fn format_tree(repo: &Repository, tree: &Tree, level: usize) -> String {
    let mut result = String::new();
    for entry in tree.iter() {
        for _ in 0..level {
            result.push_str("  ");
        }
        result.push_str(&format!("{}\n", entry.name().unwrap_or("")));
        if let Ok(subtree) = entry.to_object(repo).and_then(|obj| obj.peel_to_tree()) {
            result.push_str(&format_tree(repo, &subtree, level + 1));
        }
    }
    result
}

pub fn get_branches(repo: &Repository) -> Result<Vec<Branch>, git2::Error> {
    let mut branches = vec![];
    let branch_iter = repo.branches(Some(BranchType::Local))?;
    for branch in branch_iter {
        let (branch, _) = branch?;
        let name = branch.name()?.unwrap_or("").to_string();
        branches.push(Branch {
            name,
            target: branch.get().peel_to_commit()?,
        });
    }
    Ok(branches)
}

pub fn display_tree(f: &mut Frame, repo: &Repository, branches: &[String]) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let mut tree_text = String::new();
    for branch in branches {
        tree_text.push_str(&format!("Branch: {}\n", branch));
        if let Ok(commit) = repo
            .revparse_single(&format!("refs/heads/{}", branch))
            .and_then(|obj| obj.peel_to_commit())
        {
            tree_text.push_str(&format_commit_tree(&commit, 0));
        }
        tree_text.push_str("\n");
    }

    let paragraph = Paragraph::new(tree_text)
        .block(Block::default().borders(Borders::ALL).title("Git Branches"))
        .style(Style::default().fg(Color::White).bg(Color::Black));

    f.render_widget(paragraph, chunks[0]);
}

fn format_commit_tree(commit: &git2::Commit, level: usize) -> String {
    let mut result = String::new();
    for _ in 0..level {
        result.push_str("  ");
    }
    result.push_str(&format!(
        "{}: {}\n",
        commit.id(),
        commit.summary().unwrap_or("")
    ));
    if let Ok(parent) = commit.parent(0) {
        result.push_str(&format_commit_tree(&parent, level + 1));
    }
    result
}
