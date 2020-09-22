use crossterm::{
    cursor, event, execute, queue,
    style::{Color, Print, PrintStyledContent, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal, ExecutableCommand,
};
use git2::{BranchType, Delta, DiffDelta, DiffOptions, Repository};
use std::error::Error;
use std::format;
use std::io::{stdout, Write};

fn get_repo_diff<'a>(
    repo: &'a Repository,
    branch_name: &str,
) -> Result<git2::Diff<'a>, git2::Error> {
    let mut diff_options = DiffOptions::default();

    repo.find_branch(branch_name, BranchType::Local)
        .map(|m| m.into_reference())
        .and_then(|r| r.peel_to_tree())
        .and_then(|t| repo.diff_tree_to_workdir(Some(&t), Some(&mut diff_options)))
}

fn format_delta(delta: DiffDelta) -> String {
    let status = delta.status();

    let status_text = match status {
        Delta::Added => "added",
        Delta::Deleted => "deleted",
        Delta::Unmodified => "unmodified",
        Delta::Modified => "modified",
        Delta::Renamed => "renamed",
        Delta::Copied => "copied",
        Delta::Ignored => "ignored",
        Delta::Untracked => "untrakced",
        Delta::Typechange => "typechange",
        Delta::Unreadable => "unreadable",
        Delta::Conflicted => "conflicted",
    };
    let file_path = delta
        .new_file()
        .path()
        .or(delta.old_file().path())
        .and_then(|p| p.to_str());

    match file_path {
        Some(path) => format!("{} {}", status_text, path),
        None => format!("{} unknown file\n", status_text),
    }
}

fn main() -> crossterm::Result<()> {
    let mut stdout = stdout();
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    let mut x = 1;
    let mut y = 1;
    queue!(stdout, cursor::MoveTo(x, y))?;

    let repo = match Repository::open(".") {
        Ok(r) => r,
        Err(_) => {
            panic!();
        }
    };

    let diff = get_repo_diff(&repo, "master");

    match diff {
        Ok(diff) => {
            for item in diff.deltas() {
                let status = format_delta(item);
                queue!(stdout, Print(status));
                y += 1;
                queue!(stdout, cursor::MoveTo(x, y));
            }
        }
        Err(e) => {
            queue!(stdout, Print(format!("{:}\n", e)));
        }
    };

    stdout.flush()?;

    Ok(())
}
