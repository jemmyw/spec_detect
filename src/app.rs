use crate::CodeRepo;
use std::path::PathBuf;

pub struct App {
    pub should_quit: bool,
    repo: CodeRepo,
}

impl App {
    pub fn new(repo: CodeRepo) -> App {
        App {
            should_quit: false,
            repo,
        }
    }

    pub fn on_file(&mut self, path: PathBuf) {}

    pub fn on_quit(&mut self) {
        self.should_quit = true;
    }
}
