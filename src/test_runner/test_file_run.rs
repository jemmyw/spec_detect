use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub enum TestStatus {
    Pending,
    Success,
    Failure(String),
}

#[derive(Debug, Clone)]
pub struct TestExample {
    pub started_at: Instant,
    pub finished_at: Option<Instant>,
    pub status: TestStatus,
    pub description: String,
}

impl TestExample {
    pub fn duration(&self) -> Option<Duration> {
        self.finished_at.map(|i| i.duration_since(self.started_at))
    }
}

#[derive(Debug, Clone)]
pub struct TestFileRun {
    pub files_tested: Vec<PathBuf>,
    pub test_file: String,
    pub started_at: Instant,
    pub finished_at: Option<Instant>,
    pub examples: Vec<TestExample>,
}

impl TestFileRun {
    pub fn duration(&self) -> Option<Duration> {
        self.finished_at.map(|i| i.duration_since(self.started_at))
    }
}

#[derive(Debug, Clone)]
pub struct TestSuite {
    pub test_file_runs: Vec<TestFileRun>,
    pub started_at: Instant,
    pub finished_at: Option<Instant>,
    pub example_count: u64,
}

impl TestSuite {
    pub fn duration(&self) -> Option<Duration> {
        self.finished_at.map(|i| i.duration_since(self.started_at))
    }
}
