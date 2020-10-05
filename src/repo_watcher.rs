mod changed_file;
mod code_repo;

use anyhow::Result;
pub use changed_file::ChangedFile;

use crate::util::path_sort;
use code_repo::CodeRepo;
use owning_ref::MutexGuardRef;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::sync::watch;

use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};

pub struct RepoWatcher {
    repo: Arc<Mutex<CodeRepo>>,
    branch: String,
}

impl RepoWatcher {
    pub fn new<P: AsRef<Path>, S: AsRef<str>>(path: P, branch: S) -> Result<Self> {
        let repo = CodeRepo::open(path)?;
        Ok(Self {
            repo: Arc::new(Mutex::new(repo)),
            branch: branch.as_ref().to_owned(),
        })
    }

    pub fn watch(
        &self,
        poll_duration: Duration,
        current_changes: bool,
    ) -> watch::Receiver<Vec<ChangedFile>> {
        let poll_duration = poll_duration.to_owned();
        let repo = Arc::clone(&self.repo);
        let branch = self.branch.clone();
        let (tx, rx) = watch::channel(vec![]);

        thread::spawn(move || {
            let watch = RepoWatch {
                repo,
                branch,
                poll_duration,
                tx,
            };
            watch.watch_loop(current_changes).unwrap();
        });

        rx
    }
}

pub struct RepoWatch {
    repo: Arc<Mutex<CodeRepo>>,
    branch: String,
    poll_duration: Duration,
    tx: watch::Sender<Vec<ChangedFile>>,
}

impl RepoWatch {
    fn checkout_repo<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&CodeRepo) -> (),
    {
        let repo = MutexGuardRef::new(self.repo.lock().unwrap());
        f(repo.as_ref());
        Ok(())
    }

    fn watch_loop(&self, current_changes: bool) -> Result<()> {
        let (w_tx, w_rx) = mpsc::channel::<DebouncedEvent>();
        let mut seen_files: BTreeSet<ChangedFile> = BTreeSet::new();
        let mut prefix: PathBuf = PathBuf::new();
        let mut first_changed_files = vec![];

        self.checkout_repo(|r| {
            prefix = r.path().unwrap();
            first_changed_files = r.all_changed_files(&self.branch);
        })?;

        first_changed_files.sort_unstable_by(|a, b| path_sort::mtime_comparator(&a.path, &b.path));

        if current_changes {
            self.tx.broadcast(first_changed_files.clone())?;
        }

        for file in first_changed_files.into_iter() {
            seen_files.insert(file);
        }

        let mut watcher = watcher(w_tx, Duration::from_millis(100)).unwrap();
        for file in seen_files.iter() {
            watcher.watch(file.path.to_owned(), RecursiveMode::NonRecursive)?;
        }

        loop {
            let inform_files = match w_rx.recv_timeout(self.poll_duration) {
                Ok(event) => match event {
                    DebouncedEvent::Write(path) => vec![ChangedFile::new(
                        path.strip_prefix(&prefix).unwrap().to_owned(),
                    )],
                    _ => vec![],
                },
                Err(_) => {
                    let mut new_files: BTreeSet<ChangedFile> = BTreeSet::new();

                    self.checkout_repo(|r| {
                        for file in r.new_files().into_iter() {
                            new_files.insert(file);
                        }
                    })?;

                    let new_files: Vec<ChangedFile> =
                        new_files.difference(&seen_files).cloned().collect();

                    for file in new_files.clone() {
                        watcher.watch(file.path.clone(), RecursiveMode::NonRecursive)?;
                        seen_files.insert(file);
                    }

                    new_files
                }
            };

            if !inform_files.is_empty() {
                self.tx.broadcast(inform_files)?;
            }
        }
    }
}
