#![feature(const_fn)]

extern crate tokio;

mod app_state;
mod cli;
mod configuration;
mod input;
mod program;
mod repo_watcher;
mod ruby;
mod test_runner;
mod ui;
mod util;

use app_state::{AppStateManager, Event};
use configuration::Configuration;
use repo_watcher::{ChangedFile, RepoWatcher};
// use ruby::{RSpec, RSpecConfiguration};
use util::path_filter::PathFilter;

use anyhow::{Context, Result};
use program::Program;
use std::time::Duration;
use tokio::stream::StreamExt;

use state::LocalStorage;

static CONFIG: LocalStorage<Configuration> = LocalStorage::new();

#[tokio::main]
async fn main() -> Result<()> {
    let config = Configuration::read_configuration()?;
    let path_filter = PathFilter::new(&config).context("Invalid include configuration")?;

    CONFIG.set(move || config.to_owned());

    let watcher = RepoWatcher::new(".", CONFIG.get().branch.as_str())?;
    let watch_rx = watcher
        .watch(Duration::from_millis(1000), true)
        .map(move |files| {
            files
                .into_iter()
                .filter(|f| path_filter.include_path(&f.path))
                .collect::<Vec<ChangedFile>>()
        });

    let state_manager = AppStateManager::new();

    let mut ctrl_c_dispatcher = state_manager.dispatcher();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        ctrl_c_dispatcher.send(app_state::Event::Quit).await
    });

    let mut files_dispatcher = state_manager.dispatcher();
    tokio::spawn(async move {
        tokio::pin!(watch_rx);
        loop {
            match watch_rx.next().await {
                Some(files) => {
                    files_dispatcher
                        .send(Event::FilesChanged(files))
                        .await
                        .unwrap();
                }
                None => {
                    break;
                }
            }
        }
    });

    // let program = ui::TuiApp {};
    let program = cli::CliApp {};
    program.run(state_manager).await
}
