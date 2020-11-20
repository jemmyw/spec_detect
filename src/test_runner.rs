mod test_map;
pub mod test_run;
mod test_suite;

use crate::ChangedFile;
use std::sync::Arc;
use test_run::{TestRun, TestRunEvent};
use test_suite::TestSuite;
use tokio::sync::mpsc;

pub struct TestRunner {
    tx: mpsc::Sender<Vec<ChangedFile>>,
    rx: Option<mpsc::Receiver<Vec<ChangedFile>>>,
    files_to_test: Vec<ChangedFile>,
    test_suite: Arc<TestSuite>,
}

impl TestRunner {
    pub fn new() -> TestRunner {
        let (tx, rx) = mpsc::channel(10);

        TestRunner {
            tx,
            rx: Some(rx),
            files_to_test: vec![],
            test_suite: Arc::new(TestSuite::new()),
        }
    }

    pub fn dispatcher(&self) -> mpsc::Sender<Vec<ChangedFile>> {
        self.tx.clone()
    }

    pub fn run(&mut self) -> anyhow::Result<mpsc::Receiver<TestRunEvent>> {
        let mut rx = self.rx.take().unwrap();
        let (r_tx, r_rx) = mpsc::channel(10);
        let suite = self.test_suite.clone();

        tokio::spawn(async move {
            let mut test_run: Option<TestRun> = None;

            loop {
                let suite = Arc::clone(&suite);
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
                        test_run = TestRun::run(suite, files, r_tx).ok();
                    }
                }
            }
        });

        Ok(r_rx)
    }
}
