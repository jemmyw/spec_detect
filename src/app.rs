use crate::repo_watcher::ChangedFile;
use crate::util::path_filter::PathFilter;

use std::collections::BTreeSet;
use std::path::Path;
use std::{fs, io};

pub struct App {
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
    pub fn new(path_filter: PathFilter) -> App {
        let mut app = App {
            path_filter,
            should_quit: false,
            changed_files: vec![],
        };

        app
    }

    pub fn on_file_event(&mut self, event: Vec<ChangedFile>) -> anyhow::Result<()> {
        let mut set: BTreeSet<ChangedFile> = BTreeSet::new();

        for file in event
            .into_iter()
            .chain(self.changed_files.clone().into_iter())
        {
            set.insert(file);
        }

        let set: Vec<ChangedFile> = set.into_iter().collect();
        self.changed_files = sort_changed_files(&set);

        Ok(())
    }

    pub fn on_quit(&mut self) {
        self.should_quit = true;
    }
}
