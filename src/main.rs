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
use repo_watcher::RepoWatcher;
// use ruby::{RSpec, RSpecConfiguration};
use util::path_filter::PathFilter;

use anyhow::{Context, Result};
use std::io;
use std::time::Duration;
use tokio;

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

    let watcher = RepoWatcher::new(".", "master")?;
    let mut watch_rx = watcher.watch(Duration::from_millis(250), true);
    dbg!(watch_rx.recv().await.unwrap());
    let mut input_rx = input::listen();
    input_rx.recv().await.unwrap();

    let mut app = App::new(path_filter);

    loop {
        dbg!("before draw");
        terminal
            .draw(|f| ui::draw(f, &mut app))
            .context("Error while updating UI")?;

        tokio::select! {
            files = watch_rx.recv() => {
                let files = files.unwrap();
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
