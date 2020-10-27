#![feature(const_fn)]

extern crate tokio;

use structopt::StructOpt;
mod app_state;
mod cli;
mod configuration;
mod input;
mod program;
mod repo_watcher;
mod ruby;
mod some_loop;
mod test_runner;
mod ui;
mod util;

use app_state::{AppStateManager, Event};
use configuration::Configuration;
use repo_watcher::{ChangedFile, RepoWatcher};
// use ruby::{RSpec, RSpecConfiguration};
use test_runner::{TestEvent, TestRunner};
use util::path_filter::PathFilter;

use anyhow::{Context, Result};
use program::Program;
use std::time::Duration;
use tokio::stream::{Stream, StreamExt};

use state::LocalStorage;

static CONFIG: LocalStorage<Configuration> = LocalStorage::new();

fn watch_repo(
    branch: &str,
    path_filter: PathFilter,
) -> Result<impl Stream<Item = Vec<ChangedFile>>> {
    let watcher = RepoWatcher::new(".", branch)?;
    Ok(watcher
        .watch(Duration::from_millis(1000), true)
        .map(move |files| {
            files
                .into_iter()
                .filter(|f| path_filter.include_path(&f.path))
                .collect::<Vec<ChangedFile>>()
        }))
}

fn program_from_opt(opt: &program::Opt) -> Box<dyn Program> {
    if opt.cli {
        Box::new(cli::CliApp {})
    } else {
        Box::new(ui::TuiApp {})
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = program::Opt::from_args();
    let config = Configuration::read_configuration()?;
    let path_filter = PathFilter::new(&config).context("Invalid include configuration")?;

    CONFIG.set(move || config.to_owned());

    let changed_files_stream = watch_repo(CONFIG.get().branch.as_str(), path_filter)?;
    let state_manager = AppStateManager::new();
    let mut test_runner = TestRunner::new();

    let mut ctrl_c_dispatcher = state_manager.dispatcher();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        ctrl_c_dispatcher.send(app_state::Event::Quit).await
    });

    let mut files_dispatcher = state_manager.dispatcher();
    let mut test_files_dispatcher = test_runner.dispatcher();
    tokio::spawn(async move {
        tokio::pin!(changed_files_stream);

        some_loop!(files = changed_files_stream.next() => {
            files_dispatcher
                .send(Event::FilesChanged(files.clone()))
                .await
                .unwrap();
            test_files_dispatcher.send(files).await.unwrap();
        });
    });

    let test_dispatcher = state_manager.dispatcher();
    let test_event_stream = test_runner.run()?;
    tokio::spawn(async move {
        tokio::pin!(test_event_stream);

        some_loop!(event = test_event_stream.next() => {
            match event {
                TestEvent::TestRunning(group) => {

                }
                TestEvent::TestProgress(progress) => {

                }
                TestEvent::TestPassed(progress) => {

                }
                TestEvent::TestFailed(progress) => {}
            }
        });
    });

    let program = program_from_opt(&opt);
    program.run(state_manager).await
}
