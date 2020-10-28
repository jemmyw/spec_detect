use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    rc::Rc,
};

use crate::test_runner::TestEvent;
use crate::ChangedFile;
use crate::CONFIG;
use anyhow::{anyhow, Context};
use lazy_static::lazy_static;
use regex::Regex;
use tokio::sync::mpsc;

pub struct TestRun {
    changed_files: Vec<ChangedFile>,
    tx: mpsc::Sender<TestEvent>,
}

lazy_static! {
    static ref NUM_PATTERN: Regex = Regex::new(r"[^\\](\$(?P<num>\d+))").unwrap();
}

fn replace_matches(result_string: &String, matches: Vec<Option<&str>>) -> String {
    let mut output = result_string.clone();

    for m in NUM_PATTERN.captures_iter(result_string) {
        if let Some(group) = m.get(1) {
            let n = m.name("num").unwrap().as_str().parse::<usize>().unwrap();
            assert!(n > 0);
            let s = matches.get(n - 1).unwrap_or(&None).unwrap_or("");
            output = output.replace(group.as_str(), s);
        }
    }

    output
}

fn pattern_matches(pattern: &Regex, result_string: &String, input: &String) -> Option<String> {
    match pattern.captures(input) {
        Some(captures) => {
            let matches = captures
                .iter()
                .skip(1)
                .map(|m| m.map(|m| m.as_str()))
                .collect::<Vec<Option<&str>>>();

            Some(replace_matches(result_string, matches))
        }
        None => None,
    }
}

fn test_files_for_file(test_map: &Vec<(Regex, String)>, file: String) -> Vec<String> {
    test_map
        .iter()
        .filter_map(|(r, s)| pattern_matches(r, s, &file))
        .collect()
}

impl TestRun {
    pub fn run<'a>(
        changed_files: Vec<ChangedFile>,
        tx: mpsc::Sender<TestEvent>,
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

        let mut globs: Vec<Rc<String>> = vec![];
        let mut glob_map: HashMap<Rc<String>, Vec<&ChangedFile>> =
            HashMap::with_capacity(changed_files.len());
        let mut seen_globs: HashSet<Rc<String>> = HashSet::with_capacity(globs.len());

        for cf in changed_files.iter() {
            let path_string = cf.path.to_string_lossy().to_string();
            for test_file_glob in test_files_for_file(&test_map, path_string) {
                let test_file_glob = Rc::new(test_file_glob);
                seen_globs.insert(Rc::clone(&test_file_glob));

                match glob_map.get_mut(&test_file_glob) {
                    Some(changed_files) => {
                        changed_files.push(cf);
                    }
                    None => {
                        glob_map.insert(Rc::clone(&test_file_glob), vec![&cf]);
                    }
                }

                globs.push(Rc::clone(&test_file_glob));
            }
        }

        let mut test_files: Vec<Rc<PathBuf>> = vec![];
        let mut test_file_map: HashMap<Rc<PathBuf>, Vec<&ChangedFile>> =
            HashMap::with_capacity(glob_map.len());

        Ok(Self { changed_files, tx })
    }

    pub fn cancel(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_matches() {
        let result_string = "test/$1/$2_spec.rb".to_string();

        assert_eq!(
            replace_matches(&result_string, vec![Some("models"), Some("user")]),
            "test/models/user_spec.rb"
        );

        assert_eq!(
            replace_matches(&result_string, vec![None, Some("user")]),
            "test//user_spec.rb"
        );

        assert_eq!(
            replace_matches(&result_string, vec![Some("models")]),
            "test/models/_spec.rb"
        );
    }

    #[test]
    fn test_pattern_matches() {
        let pattern = Regex::new(r"app/(.+?)/(.+?).rb").unwrap();
        let result_string = "test/$1/$2_spec.rb".to_string();

        assert_eq!(
            pattern_matches(&pattern, &result_string, &"app/models/user.rb".to_string()),
            Some("test/models/user_spec.rb".to_string())
        );

        assert_eq!(
            pattern_matches(&pattern, &result_string, &"app/user.rb".to_string()),
            None
        );
    }

    #[test]
    fn test_test_files_for_file() {
        let test_map = vec![
            (
                Regex::new(r"app/(.+?)/(.+?).rb").unwrap(),
                "test/$1/$2_spec.rb".to_string(),
            ),
            (
                Regex::new(r"app/models/(.+?).rb").unwrap(),
                "test/*/*$1*_spec.rb".to_string(),
            ),
            (
                Regex::new("lib/(.+?).rb").unwrap(),
                "test/lib/$1.rb".to_string(),
            ),
        ];

        assert_eq!(
            test_files_for_file(&test_map, "app/controllers/users_controller.rb".to_string()),
            vec!["test/controllers/users_controller_spec.rb".to_string()]
        );

        assert_eq!(
            test_files_for_file(&test_map, "app/models/user.rb".to_string()),
            vec!["test/models/user_spec.rb", "test/*/*user*_spec.rb"]
        );

        assert_eq!(
            test_files_for_file(&test_map, "lib/simple.rb".to_string()),
            vec!["test/lib/simple.rb"]
        )
    }
}
