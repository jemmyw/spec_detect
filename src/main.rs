mod app;
mod code_repo;
mod event;
mod ruby;
mod ui;

use app::App;
use code_repo::CodeRepo;
use event::{Config, Event, Events};
use ruby::RSpec;

use anyhow::{Context, Result};
use std::io;

use termion::event::Key;
use termion::raw::IntoRawMode;

use tui::backend::TermionBackend;
use tui::Terminal;

fn main() -> Result<()> {
    let stdout = io::stdout()
        .into_raw_mode()
        .context("Could not open stdout")?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Could not create a terminal")?;
    terminal.clear().context("Could not clear the terminal")?;

    let repo = CodeRepo::open(".").context("Could not open git repository in .")?;

    let mut config = Config::default();
    config.paths = vec!["src".to_owned()];
    let events = Events::with_config(config);
    let mut app = App::new(repo, "master");

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
