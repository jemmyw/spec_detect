use git2::Delta;
// use std::ffi::CString;
use std::cmp::Ordering;
use std::path::PathBuf;

#[derive(Clone, Debug, Eq)]
pub struct ChangedFile {
    pub path: PathBuf,
    pub status: Delta,
}

impl Ord for ChangedFile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path.cmp(&other.path)
    }
}

impl PartialOrd for ChangedFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ChangedFile {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}

impl ChangedFile {
    pub fn new(path: PathBuf) -> Self {
        ChangedFile {
            path,
            status: Delta::Modified,
        }
    }
}
