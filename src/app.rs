use crate::code_repo::{ChangedFile, CodeRepo};
use crate::event::DebouncedEvent;
use git2::Delta;
use std::error::Error;
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
    pub fn new(repo: CodeRepo, branch_name: &str) -> App {
        let mut r = repo;
        let changed_files = sort_changed_files(&r.changed_files(branch_name));

        App {
            should_quit: false,
            repo: r,
            changed_files,
        }
    }

    fn prefix(&self) -> PathBuf {
        self.repo.path().unwrap()
    }

    fn add_changed_file(&mut self, p: PathBuf, delta: Delta) -> Result<&Self, Box<dyn Error>> {
        self.remove_changed_file(&p);

        match p.strip_prefix(self.prefix()) {
            Ok(path) => {
                let changed_file = ChangedFile {
                    path: path.to_path_buf(),
                    status: delta,
                };

                self.changed_files.push(changed_file);
                self.changed_files = sort_changed_files(&self.changed_files);
                Ok(self)
            }
            Err(e) => Err(Box::from(e)),
        }
    }

    fn remove_changed_file(&mut self, p: &PathBuf) {
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
    }

    pub fn on_file_event(&mut self, event: DebouncedEvent) {
        match event {
            DebouncedEvent::Create(f) => {
                self.add_changed_file(f, Delta::Added);
            }
            DebouncedEvent::Write(f) | DebouncedEvent::NoticeWrite(f) => {
                self.add_changed_file(f, Delta::Modified);
            }
            DebouncedEvent::Remove(f) => {
                self.remove_changed_file(&f);
            }
            _ => {}
        }
    }

    pub fn on_quit(&mut self) {
        self.should_quit = true;
    }
}
