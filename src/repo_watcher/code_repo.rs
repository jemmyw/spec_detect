use git2::{BranchType, Delta, DiffOptions, Repository, Status, StatusEntry, StatusOptions};
// use std::ffi::CString;
use crate::repo_watcher::ChangedFile;
use std::path::{Path, PathBuf};

fn status_to_changed_file(status: StatusEntry, delta: Delta) -> Option<ChangedFile> {
    status.path().map(|p| ChangedFile {
        path: PathBuf::from(p),
        status: delta,
    })
}

pub struct CodeRepo {
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

    pub fn path(&self) -> Option<PathBuf> {
        self.repo
            .path()
            .canonicalize()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
    }

    pub fn new_files(&self) -> Vec<ChangedFile> {
        let mut status_options = StatusOptions::default();
        status_options.include_untracked(true);

        let r = &self.repo;

        let statuses = r.statuses(Some(&mut status_options)).unwrap();
        let from_status = statuses.iter().filter_map(|s| match s.status() {
            Status::WT_NEW => status_to_changed_file(s, Delta::Added),
            Status::INDEX_NEW => status_to_changed_file(s, Delta::Added),
            Status::WT_MODIFIED => status_to_changed_file(s, Delta::Modified),
            _ => None,
        });

        let new_files: Vec<ChangedFile> = from_status.collect();

        new_files
    }

    pub fn changed_files(&self, branch_name: &str) -> Vec<ChangedFile> {
        let mut diff_options = DiffOptions::default();

        let r = &self.repo;

        let diff = r
            .find_branch(branch_name, BranchType::Local)
            .map(|m| m.into_reference())
            .and_then(|r| r.peel_to_tree())
            .and_then(|t| r.diff_tree_to_workdir_with_index(Some(&t), Some(&mut diff_options)))
            .unwrap();

        diff.deltas()
            .filter_map(|delta| {
                let status = delta.status();

                match status {
                    Delta::Deleted => None,
                    Delta::Unmodified => None,
                    Delta::Ignored => None,
                    Delta::Unreadable => None,
                    _ => delta.new_file().path().map(|p| ChangedFile {
                        path: p.to_path_buf(),
                        status,
                    }),
                }
            })
            .collect()
    }

    pub fn all_changed_files(&self, branch_name: &str) -> Vec<ChangedFile> {
        let mut files = self
            .new_files()
            .into_iter()
            .chain(self.changed_files(branch_name).into_iter())
            .collect::<Vec<ChangedFile>>();
        files.sort_unstable();
        files.dedup();
        return files;
    }
}
