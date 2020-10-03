#![feature(const_fn)]

mod app;
mod configuration;
mod event;
mod repo_watcher;
mod ruby;
mod ui;
mod util;

use app::App;
use configuration::Configuration;
use event::{Event, Events};
use repo_watcher::CodeRepo;
use ruby::{RSpec, RSpecConfiguration};
use util::path_filter::PathFilter;

use anyhow::{Context, Result};
use std::io;

use termion::event::Key;
use termion::raw::IntoRawMode;

use tui::backend::TermionBackend;
use tui::Terminal;

use state::LocalStorage;

static CONFIG: LocalStorage<Configuration> = LocalStorage::new();

fn main() -> Result<()> {
    let config = Configuration::read_configuration()?;
    let path_filter = PathFilter::new(&config).context("Invalid include configuration")?;

    CONFIG.set(move || config.to_owned());

    let stdout = io::stdout()
        .into_raw_mode()
        .context("Could not open stdout")?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Could not create a terminal")?;
    terminal.clear().context("Could not clear the terminal")?;

    let repo = CodeRepo::open(".").context("Could not open git repository in .")?;
    let events = Events::new()?;
    let mut app = App::new(repo, "master", path_filter);

    loop {
        terminal
            .draw(|f| ui::draw(f, &mut app))
            .context("Error while updating UI")?;

        match events.next()? {
            Event::Input(key) => match key {
                Key::Char('q') => {
                    dbg!("quit");
                    app.on_quit();
                }
                Key::Char(c) => {
                    dbg!(c);
                }
                _ => {}
            },
            Event::File(event) => {
                app.on_file_event(event).ok();
            }
            _ => {}
        }

        if app.should_quit {
            break;
        }
    }

    terminal.clear().ok();

    Ok(())
}
