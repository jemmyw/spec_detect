mod test_file_run;
mod test_run;

use crate::ChangedFile;
use std::{path::PathBuf, time::Instant};
use test_run::TestRun;
use tokio::sync::mpsc;

pub struct TestRunner {
    tx: mpsc::Sender<Vec<ChangedFile>>,
    rx: Option<mpsc::Receiver<Vec<ChangedFile>>>,
    files_to_test: Vec<ChangedFile>,
}

#[derive(Debug, Clone)]
pub struct TestGroup {
    changed_files: Vec<ChangedFile>,
    test_files: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct TestProgress {
    file_being_tested: ChangedFile,
    test_file: String,
    suite_start: Instant,
    file_start: Instant,
}

#[derive(Debug, Clone)]
pub enum TestEvent {
    TestRunning(TestGroup),
    TestProgress(TestProgress),
    TestPassed(TestProgress),
    TestFailed(TestProgress),
}

impl TestRunner {
    pub fn new() -> TestRunner {
        let (tx, rx) = mpsc::channel(10);

        TestRunner {
            tx,
            rx: Some(rx),
            files_to_test: vec![],
        }
    }

    pub fn dispatcher(&self) -> mpsc::Sender<Vec<ChangedFile>> {
        self.tx.clone()
    }

    pub fn run(&mut self) -> anyhow::Result<mpsc::Receiver<TestEvent>> {
        let mut rx = self.rx.take().unwrap();
        let (r_tx, r_rx) = mpsc::channel(10);

        tokio::spawn(async move {
            let mut test_run: Option<TestRun> = None;

            loop {
                let changed_files = rx.recv().await;

                if test_run.is_some() {
                    test_run.take().unwrap().cancel();
                }

                match changed_files {
                    None => {
                        break;
                    }
                    Some(files) => {
                        let r_tx = r_tx.clone();
                        test_run = TestRun::run(files, r_tx).ok();
                    }
                }
            }
        });

        Ok(r_rx)
    }
}
