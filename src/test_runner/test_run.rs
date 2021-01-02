use std::{path::PathBuf, time::Instant};

use crate::ChangedFile;
use crate::CONFIG;
use crate::{
    ruby::rspec::{RSpec, RSpecEvent, RSpecRun},
    test_runner::test_map::*,
};

use anyhow::{anyhow, Context};
use regex::Regex;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct TestRunExample {
    path: PathBuf,
    location: String,
}

#[derive(Debug)]
pub struct TestRunFailure {
    example: TestRunExample,
    description: String,
}

#[derive(Debug)]
pub enum TestRunEvent {
    Started,
    FileStarted(PathBuf),
    FileFinished(PathBuf),
    ExamplePassed(TestRunExample),
    ExampleFailed(TestRunFailure),
    Finished,
    Error(String),
}

pub struct TestRun {
    pub rspec_run: RSpecRun,
}

impl TestRun {
    pub fn run(
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
        dbg!(&config);
        let rspec = RSpec::new(config);
        let locations = file_groups
            .list
            .iter()
            .filter_map(|f| f.as_os_str().to_str())
            .collect::<Vec<&str>>();
        let (rtx, mut rrx) = mpsc::channel::<RSpecEvent>(1);

        let tx = tx.clone();

        tokio::spawn(async move {
            loop {
                let event = rrx.recv().await;

                match event {
                    None => {
                        tx.send(TestRunEvent::Error("Events stopped".to_string()))
                            .await
                            .unwrap();
                        break;
                    }
                    Some(event) => {
                        let mut current_file_started_at = Instant::now();
                        let mut current_file: Option<PathBuf> = None;

                        match event.clone() {
                            RSpecEvent::Start { .. } => {
                                tx.send(TestRunEvent::Started).await.unwrap()
                            }
                            RSpecEvent::ExampleStarted {
                                file_path: new_file_path,
                                ..
                            } => match current_file {
                                None => {
                                    current_file_started_at = Instant::now();

                                    current_file.replace(new_file_path.clone());
                                    tx.send(TestRunEvent::FileStarted(new_file_path))
                                        .await
                                        .unwrap();
                                }
                                Some(current_file_path) => {
                                    if current_file_path != new_file_path {
                                        current_file_started_at = Instant::now();
                                        current_file = Some(new_file_path.clone());

                                        tx.send(TestRunEvent::FileFinished(current_file_path))
                                            .await
                                            .unwrap();
                                        tx.send(TestRunEvent::FileStarted(new_file_path))
                                            .await
                                            .unwrap();
                                    }
                                }
                            },
                            RSpecEvent::ExamplePassed {
                                location,
                                file_path,
                                description,
                                run_time,
                                ..
                            } => {
                                tx.send(TestRunEvent::ExamplePassed(TestRunExample {
                                    location,
                                    path: file_path,
                                }))
                                .await
                                .unwrap();
                            }
                            RSpecEvent::ExampleFailed {
                                location,
                                file_path,
                                description,
                                run_time,
                                exception,
                                ..
                            } => {
                                tx.send(TestRunEvent::ExampleFailed(TestRunFailure {
                                    description: exception.unwrap_or_default(),
                                    example: TestRunExample {
                                        location: location.unwrap_or_default(),
                                        path: file_path,
                                    },
                                }))
                                .await
                                .unwrap();
                            }
                            RSpecEvent::Stop {} => {
                                tx.send(TestRunEvent::Finished).await.unwrap();
                            }
                            RSpecEvent::Error { msg } => {
                                tx.send(TestRunEvent::Error(msg)).await.unwrap();
                                break;
                            }
                            RSpecEvent::Exit => {
                                tx.send(TestRunEvent::Finished).await.unwrap();
                                break;
                            }
                        }
                    }
                }
            }
        });

        let rspec_run = rspec.run(locations.clone(), rtx)?;

        Ok(Self { rspec_run })
    }

    pub fn cancel(&mut self) -> anyhow::Result<()> {
        self.rspec_run.kill()
    }
}
