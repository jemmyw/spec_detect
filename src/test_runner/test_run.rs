use crate::test_runner::TestEvent;
use crate::ChangedFile;
use crate::CONFIG;
use anyhow::{anyhow, Context};
use regex::Regex;
use tokio::sync::mpsc;

pub struct TestRun {
    changed_files: Vec<ChangedFile>,
    tx: mpsc::Sender<TestEvent>,
}

fn replace_matches(result_string: &String, matches: Vec<Option<&str>>) -> String {
    let mut output = result_string.clone();
    let r = Regex::new(r"[^\\](\$(?P<num>\d+))").unwrap();

    for m in r.captures_iter(result_string) {
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

impl TestRun {
    pub fn run(
        changed_files: Vec<ChangedFile>,
        tx: mpsc::Sender<TestEvent>,
    ) -> anyhow::Result<Self> {
        let test_map = CONFIG
            .get()
            .map
            .get("rspec")
            .ok_or_else(|| anyhow!("No rspec in test mappings"))?
            .into_iter()
            .map(|(pat_string, res_string)| Regex::new(pat_string).map(|r| (r, res_string)))
            .collect::<Result<Vec<_>, _>>()
            .context("Invalid regex in test map")?
            .into_iter();

        // changed_files.iter().map(|cf| for (p, r) in test_map {});

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
}
