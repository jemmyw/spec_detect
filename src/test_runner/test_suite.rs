use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{path::PathBuf, time::Duration};

use anyhow::Result;

#[derive(Debug, Clone)]
pub enum TestStatus {
    Pending,
    Success,
    Failure(String),
}

impl Default for TestStatus {
    fn default() -> Self {
        Self::Pending
    }
}

#[derive(Debug, Clone, Default)]
pub struct TestExample {
    pub status: TestStatus,
    pub description: String,
    pub failure: Option<String>,
    pub duration: Duration,
}

#[derive(Debug, Clone, Default)]
pub struct TestFile {
    pub files_tested: Vec<PathBuf>,
    pub path: PathBuf,
    pub duration: Duration,
    pub examples: Vec<TestExample>,
}

impl TestFile {
    pub fn new(path: &PathBuf) -> Self {
        let mut test_file = Self::default();
        test_file.path = path.clone();
        test_file
    }
}

impl PartialEq for TestFile {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}
impl Eq for TestFile {}
impl Hash for TestFile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

#[derive(Debug)]
pub struct TestSuite {
    test_files: RwLock<HashMap<PathBuf, TestFile>>,
}

impl TestSuite {
    pub fn new() -> Self {
        Self {
            test_files: RwLock::new(HashMap::new()),
        }
    }

    pub fn update<F>(&self, f: F) -> ()
    where
        F: FnOnce(RwLockWriteGuard<HashMap<PathBuf, TestFile>>),
    {
        let lock = self.test_files.write().unwrap();
        f(lock);
    }

    pub fn reader<F>(&self, f: F) -> ()
    where
        F: FnOnce(RwLockReadGuard<HashMap<PathBuf, TestFile>>),
    {
        let lock = self.test_files.read().unwrap();
        f(lock);
    }
}
