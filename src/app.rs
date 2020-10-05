use crate::repo_watcher::ChangedFile;

pub struct App {
    pub should_quit: bool,
    pub changed_files: Vec<ChangedFile>,
    pub last_changed_files: Vec<ChangedFile>,
}

impl App {
    pub fn new() -> App {
        App {
            should_quit: false,
            changed_files: vec![],
            last_changed_files: vec![],
        }
    }

    pub fn on_file_event(&mut self, event: Vec<ChangedFile>) -> anyhow::Result<()> {
        self.last_changed_files = event.clone();

        self.changed_files = event
            .into_iter()
            .chain(
                self.changed_files
                    .clone()
                    .into_iter()
                    .filter(|f| !self.last_changed_files.contains(f)),
            )
            .collect();

        Ok(())
    }

    pub fn on_quit(&mut self) {
        self.should_quit = true;
    }
}
