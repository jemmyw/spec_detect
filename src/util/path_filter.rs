use crate::Configuration;
use globber::Pattern;
use std::path::Path;

pub struct PathFilter {
    include_patterns: Vec<Pattern>,
}

impl PathFilter {
    pub fn new(config: &Configuration) -> anyhow::Result<Self> {
        let patterns: Result<Vec<Pattern>, globber::Error> = config
            .include
            .clone()
            .iter()
            .map(|s| Pattern::new(s))
            .collect();

        patterns
            .map(|patterns| PathFilter {
                include_patterns: patterns,
            })
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub fn include_path<T: AsRef<Path>>(&self, path: T) -> bool {
        match path.as_ref().to_str() {
            Some(s) => self.include_patterns.iter().any(|p| p.matches(s)),
            None => false,
        }
    }
}
