use std::{path::PathBuf, sync::Arc, time::Instant};

use crate::ChangedFile;
use crate::CONFIG;
use crate::{
    ruby::rspec::{RSpec, RSpecEvent, RSpecRun},
    test_runner::test_map::*,
    test_runner::test_suite::{TestExample, TestFile, TestStatus, TestSuite},
};

use anyhow::{anyhow, Context};
use regex::Regex;
use std::sync::mpsc::channel;
use tokio::sync::mpsc;

pub struct TestRunExample {
    path: PathBuf,
    location: String,
    name: String,
}

pub struct TestRunFailure {
    example: TestRunExample,
    description: String,
}

pub enum TestRunEvent {
    Started,
    FileStarted(PathBuf),
    FileFinished(PathBuf),
    ExampleStarted(TestRunExample),
    ExamplePassed(TestRunExample),
    ExampleFailed(TestRunFailure),
    Finished,
}

pub struct TestRun {
    pub rspec_run: RSpecRun,
}

impl TestRun {
    pub fn run(
        suite: Arc<TestSuite>,
        changed_files: Vec<ChangedFile>,
        tx: mpsc::Sender<TestRunEvent>,
    ) -> anyhow::Result<Self> {
        let test_map = CONFIG
            .get()
            .map
            .get("rspec")
            .ok_or_else(|| anyhow!("No rspec in test mappings"))?
            .into_iter()
            .map(|(pat_string, res_string)| Regex::new(pat_string).map(|r| (r, res_string.clone())))
            .collect::<Result<Vec<_>, _>>()
            .context("Invalid regex in test map")?;

        let glob_groups = grouped_globs(&test_map, changed_files);
        let file_groups = grouped_files(glob_groups);

        let config = CONFIG.get().rspec.clone();
        let rspec = RSpec::new(config);
        let locations = file_groups
            .list
            .iter()
            .filter_map(|f| f.as_os_str().to_str())
            .collect::<Vec<&str>>();
        let (rtx, rrx) = channel::<RSpecEvent>();

        let file_map = file_groups.map.clone();
        let mut tx = tx.clone();

        std::thread::spawn(move || loop {
            let mut last_event: Option<RSpecEvent> = None;
            let event = rrx.recv().unwrap();

            let mut current_file_started_at = Instant::now();
            let mut current_file: Option<PathBuf> = None;
            let mut current_example: Option<String> = None;

            match event.clone() {
                RSpecEvent::Start { count } => if let Some(count) = count {},
                RSpecEvent::ExampleStarted {
                    file_path: new_file_path,
                    ..
                } => match current_file {
                    None => {
                        current_file_started_at = Instant::now();
                        let test_file = TestFile::new(&new_file_path);

                        suite.update(|mut s| {
                            s.insert(new_file_path.clone(), test_file);
                        });

                        current_file.replace(new_file_path.clone());
                        tx.send(TestRunEvent::FileStarted(new_file_path));
                    }
                    Some(current_file_path) => {
                        if current_file_path != new_file_path {
                            let test_file = TestFile::new(&new_file_path);

                            suite.update(|mut s| {
                                if let Some(current_test_file) = s.get_mut(&current_file_path) {
                                    current_test_file.duration =
                                        Instant::now().duration_since(current_file_started_at);
                                }

                                s.insert(new_file_path.clone(), test_file);
                            });

                            current_file_started_at = Instant::now();
                            current_file = Some(new_file_path.clone());

                            tx.send(TestRunEvent::FileFinished(current_file_path));
                            tx.send(TestRunEvent::FileStarted(new_file_path));
                        }
                    }
                },
                RSpecEvent::ExamplePassed {
                    id,
                    load_time,
                    location,
                    file_path,
                    description,
                    run_time,
                } => {}
                RSpecEvent::ExampleFailed {
                    id,
                    load_time,
                    location,
                    file_path,
                    description,
                    run_time,
                    exception,
                } => {}
                RSpecEvent::Stop {} => {}
                RSpecEvent::Error { msg } => {
                    break;
                }
                RSpecEvent::Exit => {
                    break;
                }
            }

            last_event = Some(event);
        });

        let rspec_run = rspec.run(locations.clone(), rtx)?;

        Ok(Self { rspec_run })
    }

    pub fn cancel(&mut self) -> anyhow::Result<()> {
        self.rspec_run.kill()
    }
}
