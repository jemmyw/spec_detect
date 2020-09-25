use crate::code_repo::{ChangedFile, CodeRepo};
use std::path::{Path, PathBuf};
use std::{fs, io};

pub struct App {
    pub should_quit: bool,
    repo: CodeRepo,
    pub changed_files: Vec<ChangedFile>,
}

fn modified<P: AsRef<Path>>(path: P) -> io::Result<std::time::SystemTime> {
    fs::metadata(path).and_then(|m| m.modified())
}

fn sort_changed_files(files: Vec<ChangedFile>) -> Vec<ChangedFile> {
    let mut unsorted = files.to_vec();
    unsorted.sort_by(|a, b| {
        modified(a.path.to_path_buf())
            .ok()
            .zip(modified(b.path.to_path_buf()).ok())
            .map_or(std::cmp::Ordering::Greater, |(fa, fb)| fb.cmp(&fa))
    });
    unsorted
}

impl App {
    pub fn new(repo: CodeRepo) -> App {
        let mut r = repo;
        let changed_files = sort_changed_files(r.changed_files("master"));

        App {
            should_quit: false,
            repo: r,
            changed_files,
        }
    }

    fn changed_files(&mut self) -> Vec<ChangedFile> {
        sort_changed_files(self.repo.changed_files("master"))
    }

    pub fn on_file(&mut self, path: PathBuf) {
        self.changed_files = self.changed_files();
    }

    pub fn on_quit(&mut self) {
        self.should_quit = true;
    }
}
