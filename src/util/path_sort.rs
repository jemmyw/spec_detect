use std::cmp::Ordering;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn modified<P: AsRef<Path>>(path: P) -> io::Result<std::time::SystemTime> {
    fs::metadata(path).and_then(|m| m.modified())
}

pub fn mtime_comparator(a: &PathBuf, b: &PathBuf) -> Ordering {
    modified(a)
        .ok()
        .zip(modified(b).ok())
        .map_or(Ordering::Greater, |(a, b)| b.cmp(&a))
}

pub fn sort_by_mtime(files: &Vec<PathBuf>) -> Vec<PathBuf> {
    let mut unsorted = files.to_vec();
    unsorted.sort_by(mtime_comparator);
    unsorted
}
