#![feature(const_fn)]

mod app;
mod configuration;
mod input;
mod repo_watcher;
mod ruby;
mod ui;
mod util;

use app::App;
use configuration::Configuration;
use repo_watcher::{ChangedFile, RepoWatcher};
// use ruby::{RSpec, RSpecConfiguration};
use util::path_filter::PathFilter;

use anyhow::{Context, Result};
use std::io;
use std::time::Duration;
use tokio::{self, stream::StreamExt};

use termion::event::Key;
use termion::raw::IntoRawMode;

use tui::backend::TermionBackend;
use tui::Terminal;

use state::LocalStorage;

static CONFIG: LocalStorage<Configuration> = LocalStorage::new();

#[tokio::main]
async fn main() -> Result<()> {
    let config = Configuration::read_configuration()?;
    let path_filter = PathFilter::new(&config).context("Invalid include configuration")?;

    CONFIG.set(move || config.to_owned());

    let stdout = io::stdout()
        .into_raw_mode()
        .context("Could not open stdout")?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Could not create a terminal")?;
    terminal.clear().context("Could not clear the terminal")?;

    let watcher = RepoWatcher::new(".", CONFIG.get().branch.as_str())?;
    let watch_rx = watcher
        .watch(Duration::from_millis(250), true)
        .map(|files| {
            files
                .into_iter()
                .filter(|f| path_filter.include_path(&f.path))
                .collect::<Vec<ChangedFile>>()
        });
    tokio::pin!(watch_rx);

    let mut input_rx = input::listen();
    let mut app = App::new();

    loop {
        dbg!("before draw");
        terminal
            .draw(|f| ui::draw(f, &mut app))
            .context("Error while updating UI")?;

        tokio::select! {
            files = watch_rx.next() => {
                let files = files.unwrap();
                app.on_file_event(files)?;
            }
            key = input_rx.recv() => {
                let key = key.unwrap();

                match key {
                    Key::Char('q') => {
                        dbg!("quit");
                        app.on_quit();
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    terminal.clear().ok();

    Ok(())
}
