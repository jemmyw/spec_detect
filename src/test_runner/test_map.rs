use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use crate::ChangedFile;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref NUM_PATTERN: Regex = Regex::new(r"[^\\](\$(?P<num>\d+))").unwrap();
}

#[derive(Clone)]
pub struct Group<T> {
    pub list: Vec<T>,
    pub map: HashMap<T, Vec<ChangedFile>>,
}
impl<T> Group<T> {
    pub fn len(&self) -> usize {
        self.list.len()
    }
}

/**
Given a string containing any number of tokens in the form $1 $2 $3 $n, and a
vec of replacement strings, this will return a new string with the tokens
replaced. If the vec is smaller than the token number or the vec has a None
at that place then the token is just removed.
*/
fn replace_tokens(token_string: &String, replacements: Vec<Option<&str>>) -> String {
    let mut output = token_string.clone();

    for m in NUM_PATTERN.captures_iter(token_string) {
        if let Some(group) = m.get(1) {
            let n = m.name("num").unwrap().as_str().parse::<usize>().unwrap();
            assert!(n > 0);
            let s = replacements.get(n - 1).unwrap_or(&None).unwrap_or("");
            output = output.replace(group.as_str(), s);
        }
    }

    output
}

/**
Run a regex over an input string, and then for each matching group replace
tokens in a token string in the form $1 $2 $3 $n with the match.

Example:

```rust
let name = patern_matches(
    Regex::new(r"name is (.+)\b").unwrap(),
    "Hello $1",
    "My name is Bob"
);
assert_eq!(name, "Hello Bob".to_string());
```
*/
fn pattern_matches(pattern: &Regex, token_string: &String, input: &String) -> Option<String> {
    match pattern.captures(input) {
        Some(captures) => {
            let replacements = captures
                .iter()
                .skip(1)
                .map(|m| m.map(|m| m.as_str()))
                .collect::<Vec<Option<&str>>>();

            Some(replace_tokens(token_string, replacements))
        }
        None => None,
    }
}

fn test_globs_for_file(test_map: &Vec<(Regex, String)>, file: String) -> Vec<String> {
    test_map
        .iter()
        .filter_map(|(r, s)| pattern_matches(r, s, &file))
        .collect()
}

/**
Return a group where the list is the globs from the test map in the order
found using changed_files.
*/
pub fn grouped_globs(
    test_map: &Vec<(Regex, String)>,
    changed_files: Vec<ChangedFile>,
) -> Group<String> {
    let mut group = Group {
        list: Vec::with_capacity(changed_files.len()),
        map: HashMap::with_capacity(changed_files.len()),
    };
    let mut seen_globs: HashSet<String> = HashSet::with_capacity(changed_files.len());

    for cf in changed_files.iter() {
        let path_string = cf.path.to_string_lossy().to_string();
        for test_file_glob in test_globs_for_file(&test_map, path_string) {
            match group.map.get_mut(&test_file_glob) {
                Some(changed_files) => {
                    changed_files.push(cf.clone());
                }
                None => {
                    group.map.insert(test_file_glob.clone(), vec![cf.clone()]);
                }
            }

            if !seen_globs.contains(&test_file_glob) {
                seen_globs.insert(test_file_glob.clone());
                group.list.push(test_file_glob);
            }
        }
    }

    group
}

/**
Return a group where the list is actual files found on the filesystem in the
order found using the given glob group.
*/
pub fn grouped_files(globs: Group<String>) -> Group<PathBuf> {
    let mut group = Group {
        list: Vec::with_capacity(globs.len()),
        map: HashMap::with_capacity(globs.len()),
    };
    let mut seen_files: HashSet<PathBuf> = HashSet::with_capacity(globs.len());

    for glob_pattern in globs.list.iter() {
        let changed_files = globs.map.get(glob_pattern).unwrap();

        match glob::glob(glob_pattern) {
            Ok(paths) => {
                for path in paths.filter_map(|p| p.ok()) {
                    if !seen_files.contains(&path) {
                        seen_files.insert(path.clone());
                        group.list.push(path.clone());
                        group.map.insert(path.clone(), changed_files.clone());
                    } else {
                        let existing_files = group.map.get_mut(&path).unwrap();

                        for cf in changed_files {
                            existing_files.push(cf.clone());
                        }
                    }
                }
            }
            Err(_) => {}
        }
    }

    group
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_matches() {
        let result_string = "test/$1/$2_spec.rb".to_string();

        assert_eq!(
            replace_tokens(&result_string, vec![Some("models"), Some("user")]),
            "test/models/user_spec.rb"
        );

        assert_eq!(
            replace_tokens(&result_string, vec![None, Some("user")]),
            "test//user_spec.rb"
        );

        assert_eq!(
            replace_tokens(&result_string, vec![Some("models")]),
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
    fn test_test_globs_for_file() {
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
            test_globs_for_file(&test_map, "app/controllers/users_controller.rb".to_string()),
            vec!["test/controllers/users_controller_spec.rb".to_string()]
        );

        assert_eq!(
            test_globs_for_file(&test_map, "app/models/user.rb".to_string()),
            vec!["test/models/user_spec.rb", "test/*/*user*_spec.rb"]
        );

        assert_eq!(
            test_globs_for_file(&test_map, "lib/simple.rb".to_string()),
            vec!["test/lib/simple.rb"]
        )
    }

    #[test]
    fn test_grouped_globs() {
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

        let mut changed_files = vec![ChangedFile::new("app/models/user.rb")];
        let mut groups = grouped_globs(&test_map, changed_files);

        assert_eq!(groups.list.len(), 2);
        assert_eq!(
            groups.list.get(0).unwrap().as_str(),
            "test/models/user_spec.rb"
        );
        assert_eq!(
            groups.list.get(1).unwrap().as_str(),
            "test/*/*user*_spec.rb"
        );
        assert_eq!(
            groups.map.get(groups.list.get(0).unwrap()).unwrap(),
            &vec![ChangedFile::new("app/models/user.rb")]
        );
        assert_eq!(
            groups.map.get(groups.list.get(1).unwrap()).unwrap(),
            &vec![ChangedFile::new("app/models/user.rb")]
        );

        changed_files = vec![
            ChangedFile::new("app/controllers/users_controller.rb"),
            ChangedFile::new("app/models/user.rb"),
        ];
        groups = grouped_globs(&test_map, changed_files);

        assert_eq!(groups.list.len(), 3);
        assert_eq!(
            groups.list.get(0).unwrap().as_str(),
            "test/controllers/users_controller_spec.rb"
        );
        assert_eq!(
            groups.list.get(1).unwrap().as_str(),
            "test/models/user_spec.rb"
        );
        assert_eq!(
            groups.list.get(2).unwrap().as_str(),
            "test/*/*user*_spec.rb"
        );
        assert_eq!(
            groups.map.get(groups.list.get(0).unwrap()).unwrap(),
            &vec![ChangedFile::new("app/controllers/users_controller.rb")]
        );
        assert_eq!(
            groups.map.get(groups.list.get(1).unwrap()).unwrap(),
            &vec![ChangedFile::new("app/models/user.rb")]
        );
        assert_eq!(
            groups.map.get(groups.list.get(2).unwrap()).unwrap(),
            &vec![ChangedFile::new("app/models/user.rb")]
        );
    }

    #[test]
    fn test_grouped_files() {
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
        let changed_files = vec![
            ChangedFile::new("app/controllers/users_controller.rb"),
            ChangedFile::new("app/models/user.rb"),
        ];
        let groups = grouped_globs(&test_map, changed_files);
        let files = grouped_files(groups);

        assert_eq!(files.list.len(), 2);
        assert_eq!(
            files.list.get(0).unwrap().as_os_str(),
            "test/controllers/users_controller_spec.rb"
        );
        assert_eq!(
            files.list.get(1).unwrap().as_os_str(),
            "test/models/user_spec.rb"
        );

        // Test the map identifies both files for users_controller as per test/*/*user*_spec.rb
        assert!(files
            .map
            .get(files.list.get(0).unwrap())
            .unwrap()
            .contains(&ChangedFile::new("app/controllers/users_controller.rb")));
        assert!(files
            .map
            .get(files.list.get(0).unwrap())
            .unwrap()
            .contains(&ChangedFile::new("app/models/user.rb")));

        // Change order

        let changed_files = vec![
            ChangedFile::new("app/models/user.rb"),
            ChangedFile::new("app/controllers/users_controller.rb"),
        ];
        let groups = grouped_globs(&test_map, changed_files);
        let files = grouped_files(groups);

        assert_eq!(files.list.len(), 2);
        assert_eq!(
            files.list.get(0).unwrap().as_os_str(),
            "test/models/user_spec.rb"
        );
        assert_eq!(
            files.list.get(1).unwrap().as_os_str(),
            "test/controllers/users_controller_spec.rb"
        );

        // Non-existant files

        let changed_files = vec![ChangedFile::new("app/models/record.rb")];
        let groups = grouped_globs(&test_map, changed_files);
        let files = grouped_files(groups);

        assert_eq!(files.list.len(), 0);
    }
}