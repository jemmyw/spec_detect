use crate::event::DebouncedEvent;
use crate::repo_watcher::{ChangedFile, CodeRepo};
use crate::util::path_filter::PathFilter;

use git2::Delta;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub struct App {
    repo: CodeRepo,
    path_filter: PathFilter,

    pub should_quit: bool,
    pub changed_files: Vec<ChangedFile>,
}

fn modified<P: AsRef<Path>>(path: P) -> io::Result<std::time::SystemTime> {
    fs::metadata(path).and_then(|m| m.modified())
}

fn sort_changed_files(files: &Vec<ChangedFile>) -> Vec<ChangedFile> {
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
    pub fn new(repo: CodeRepo, branch_name: &str, path_filter: PathFilter) -> App {
        let mut r = repo;
        // let changed_files = sort_changed_files(&r.changed_files(branch_name));

        let mut app = App {
            repo: r,
            path_filter,
            should_quit: false,
            changed_files: vec![],
        };

        for file in app.repo.changed_files(branch_name) {
            let path = file.path.to_owned();
            dbg!(&path);
            app.add_changed_file(path, file.status);
        }

        app
    }

    fn prefix(&self) -> PathBuf {
        self.repo.path().unwrap()
    }

    fn is_relevant_file(&self, path: &Path) -> bool {
        self.path_filter.include_path(path) && path.exists() && path.is_file()
    }

    fn add_changed_file(&mut self, p: PathBuf, delta: Delta) -> anyhow::Result<&Self> {
        self.remove_changed_file(&p)?;

        let path = p.strip_prefix(self.prefix())?;

        if self.is_relevant_file(path) {
            let changed_file = ChangedFile {
                path: path.to_path_buf(),
                status: delta,
            };

            self.changed_files.push(changed_file);
            self.changed_files = sort_changed_files(&self.changed_files);
        }

        Ok(self)
    }

    fn remove_changed_file(&mut self, p: &PathBuf) -> anyhow::Result<&Self> {
        match p
            .strip_prefix(self.prefix())
            .ok()
            .and_then(|p| self.changed_files.iter().position(|f| f.path.eq(p)))
        {
            Some(index) => {
                self.changed_files.remove(index);
            }
            None => {}
        }

        Ok(self)
    }

    pub fn on_file_event(&mut self, event: DebouncedEvent) -> anyhow::Result<&Self> {
        match event {
            DebouncedEvent::Create(f) => self.add_changed_file(f, Delta::Added),
            DebouncedEvent::Write(f) | DebouncedEvent::NoticeWrite(f) => {
                self.add_changed_file(f, Delta::Modified)
            }
            DebouncedEvent::Remove(f) => self.remove_changed_file(&f),
            _ => Ok(self),
        }
    }

    pub fn on_quit(&mut self) {
        self.should_quit = true;
    }
}
