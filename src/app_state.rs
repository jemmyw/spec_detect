use crate::repo_watcher::ChangedFile;
use std::path::PathBuf;
use tokio::stream::{self, Stream, StreamExt};
use tokio::sync::{mpsc, watch};

#[derive(Debug, Clone)]
pub enum TestStatus {
    Unknown,
    Running,
    Passed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct WatchedFile {
    pub changed_file: ChangedFile,
    pub test_status: TestStatus,
}

#[derive(Debug, Clone)]
pub struct TestFile {
    pub test_file: PathBuf,
    pub changed_files: Vec<ChangedFile>,
}

#[derive(Debug, Clone)]
pub enum Event {
    Start,
    FilesChanged(Vec<ChangedFile>),
    TestRunning(TestFile),
    TestPassed(TestFile),
    TestFailed(TestFile),
    Quit,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub should_quit: bool,
    pub watched_files: Vec<WatchedFile>,
    pub last_changed_files: Vec<ChangedFile>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            should_quit: false,
            watched_files: vec![],
            last_changed_files: vec![],
        }
    }

    pub fn on(&mut self, event: Event) -> anyhow::Result<()> {
        match event {
            Event::Start => {}
            Event::FilesChanged(files) => {
                self.on_file_event(files);
            }
            Event::TestRunning(_) => {}
            Event::TestPassed(_) => {}
            Event::TestFailed(_) => {}
            Event::Quit => {
                self.on_quit();
            }
        }

        Ok(())
    }

    pub fn on_file_event(&mut self, event: Vec<ChangedFile>) -> anyhow::Result<()> {
        self.last_changed_files = event.clone();

        self.watched_files = event
            .into_iter()
            .map(|f| WatchedFile {
                changed_file: f,
                test_status: TestStatus::Unknown,
            })
            .chain(
                self.watched_files
                    .clone()
                    .into_iter()
                    .filter(|f| !self.last_changed_files.contains(&f.changed_file)),
            )
            .collect();

        Ok(())
    }

    pub fn on_quit(&mut self) {
        self.should_quit = true;
    }
}

#[derive(Clone)]
pub struct AppStateManager {
    event_tx: mpsc::Sender<Event>,
    watch_rx: watch::Receiver<(Event, AppState)>,
}

impl AppStateManager {
    pub fn new() -> AppStateManager {
        let state = AppState::new();
        let (event_tx, mut event_rx) = mpsc::channel::<Event>(10);
        let (watch_tx, watch_rx) = watch::channel((Event::Start, state));

        let mut spawn_rx = watch_rx.clone();

        tokio::spawn(async move {
            loop {
                let (_, mut state) = spawn_rx.recv().await.unwrap();
                let event = event_rx.recv().await;

                match event {
                    Some(Event::Quit) => {
                        state.on(Event::Quit);
                        watch_tx.broadcast((Event::Quit, state));
                        break;
                    }
                    Some(event) => {
                        state.on(event.clone());
                        watch_tx.broadcast((event, state));
                    }
                    None => {
                        break;
                    }
                }
            }
        });

        AppStateManager { event_tx, watch_rx }
    }

    pub async fn get_state(&self) -> Option<(Event, AppState)> {
        self.watch_rx.clone().recv().await
    }

    pub fn stream(&self) -> impl Stream<Item = (Event, AppState)> {
        self.watch_rx.clone()
    }

    pub async fn dispatch(&self, event: Event) -> anyhow::Result<()> {
        self.dispatcher()
            .send(event)
            .await
            .map_err(anyhow::Error::from)
    }

    pub fn dispatcher(&self) -> mpsc::Sender<Event> {
        self.event_tx.clone()
    }
}
