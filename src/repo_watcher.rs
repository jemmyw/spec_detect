mod changed_file;
mod code_repo;

use anyhow::Result;
pub use changed_file::ChangedFile;

use code_repo::CodeRepo;
use owning_ref::MutexGuardRef;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

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

    pub fn watch<D: AsRef<Duration>>(
        &self,
        poll_duration: D,
        current_changes: bool,
        tx: Sender<Vec<ChangedFile>>,
    ) -> Result<JoinHandle<()>> {
        let poll_duration = poll_duration.as_ref().to_owned();
        let repo = Arc::clone(&self.repo);
        let branch = self.branch.clone();

        let jh = thread::spawn(move || {
            let watch = RepoWatch {
                repo,
                branch,
                poll_duration,
                tx,
            };
            watch.watch_loop(current_changes);
        });

        Ok(jh)
    }
}

pub struct RepoWatch {
    repo: Arc<Mutex<CodeRepo>>,
    branch: String,
    poll_duration: Duration,
    tx: Sender<Vec<ChangedFile>>,
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
        let (w_tx, w_rx) = channel::<DebouncedEvent>();
        let mut seen_files: BTreeSet<ChangedFile> = BTreeSet::new();
        let mut prefix: PathBuf = PathBuf::new();

        self.checkout_repo(|r| {
            prefix = r.path().unwrap();

            for file in r.all_changed_files(&self.branch).into_iter() {
                seen_files.insert(file);
            }
        });

        if current_changes {
            self.tx.send(seen_files.clone().into_iter().collect())?;
        }

        let watcher = watcher(w_tx, Duration::from_millis(100)).unwrap();
        for file in seen_files {
            watcher.watch(file.path, RecursiveMode::NonRecursive);
        }

        loop {
            let inform_files = match w_rx.recv_timeout(self.poll_duration) {
                Ok(event) => match event {
                    DebouncedEvent::Write(path) => vec![ChangedFile::new(
                        path.strip_prefix(&prefix).unwrap().to_owned(),
                    )],
                    _ => vec![],
                },
                Err(e) => {
                    let mut new_files: BTreeSet<ChangedFile> = BTreeSet::new();

                    self.checkout_repo(|r| {
                        for file in r.new_files().into_iter() {
                            new_files.insert(file);
                        }
                    });

                    let new_files: Vec<ChangedFile> =
                        new_files.difference(&seen_files).cloned().collect();

                    for file in new_files {
                        watcher.watch(file.path, RecursiveMode::NonRecursive);
                        seen_files.insert(file);
                    }

                    new_files
                }
            };

            if !inform_files.is_empty() {
                self.tx.send(inform_files)?;
            }
        }
    }
}
