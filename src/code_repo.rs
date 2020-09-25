use git2::{BranchType, Delta, DiffOptions, Repository};
use std::ffi::CString;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct ChangedFile {
    pub path: PathBuf,
    pub status: Delta,
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

    pub fn all_files_ending(&mut self, ending: &str) -> Vec<String> {
        self.all_files()
            .into_iter()
            .filter(|s| s.ends_with(ending))
            .collect()
    }

    pub fn all_ruby_files(&mut self) -> Vec<String> {
        self.all_files_ending(".rb")
    }

    pub fn all_spec_files(&mut self) -> Vec<String> {
        self.all_files_ending("_spec.rb")
    }

    pub fn changed_files(&mut self, branch_name: &str) -> Vec<ChangedFile> {
        let mut diff_options = DiffOptions::default();
        let r = &self.repo;

        r.find_branch(branch_name, BranchType::Local)
            .map(|m| m.into_reference())
            .and_then(|r| r.peel_to_tree())
            .and_then(|t| r.diff_tree_to_workdir(Some(&t), Some(&mut diff_options)))
            .map(|diff| {
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
                    .collect::<Vec<ChangedFile>>()
            })
            .unwrap()
    }
}
