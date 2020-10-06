mod test_run;

use std::path::PathBuf;
use test_run::TestRun;
pub struct TestRunner {}

impl TestRunner {
    pub fn new() -> TestRunner {
        TestRunner {}
    }

    pub fn queue(files: Vec<PathBuf>) -> TestRun {
        TestRun {}
    }
}
