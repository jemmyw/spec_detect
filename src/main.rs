use git2::{BranchType, Delta, DiffDelta, DiffOptions, Repository};
use std::ffi::CString;
use std::io;
use std::path::Path;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::Terminal;

pub mod event;
pub mod ui;
pub mod watcher;

struct CodeRepo {
    repo: Repository,
}

impl CodeRepo {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<CodeRepo, git2::Error> {
        let repo = Repository::open(path);

        match repo {
            Ok(repo) => Ok(CodeRepo { repo }),
            Err(e) => Err(e),
        }
    }

    pub fn all_files(&mut self) -> Vec<String> {
        let index = self.repo.index().unwrap();
        index
            .iter()
            .map(|f| {
                CString::new(&f.path[..])
                    .unwrap()
                    .to_str()
                    .map(String::from)
                    .unwrap()
            })
            .collect()
    }

    pub fn all_spec_files(&mut self) -> Vec<String> {
        self.all_files()
            .into_iter()
            .filter(|s| s.ends_with("_spec.rb"))
            .collect()
    }

    pub fn changed_files(&mut self, branch_name: &str) -> Vec<String> {
        let mut diff_options = DiffOptions::default();
        let r = &self.repo;

        r.find_branch(branch_name, BranchType::Local)
            .map(|m| m.into_reference())
            .and_then(|r| r.peel_to_tree())
            .and_then(|t| r.diff_tree_to_workdir(Some(&t), Some(&mut diff_options)))
            .map(|diff| {
                diff.deltas()
                    .filter_map(|delta| match delta.status() {
                        Delta::Deleted => None,
                        Delta::Unmodified => None,
                        Delta::Ignored => None,
                        Delta::Unreadable => None,
                        _ => delta
                            .new_file()
                            .path()
                            .and_then(|p| p.to_str().map(String::from)),
                    })
                    .collect::<Vec<String>>()
            })
            .unwrap()
    }
}

fn main() -> crossterm::Result<()> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut repo = match CodeRepo::open(".") {
        Ok(r) => r,
        Err(_) => {
            panic!();
        }
    };

    Ok(())
}
