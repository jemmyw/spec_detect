mod code_repo;
mod event;
mod ui;

use code_repo::CodeRepo;
use event::{Config, Event, Events};

use std::error::Error;
use std::io;

use termion::event::Key;
use termion::raw::IntoRawMode;

use tui::backend::TermionBackend;
use tui::Terminal;

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut repo = match CodeRepo::open(".") {
        Ok(r) => r,
        Err(_) => {
            panic!();
        }
    };

    let events = Events::with_config(Config::default());
    let mut should_quit = false;

    loop {
        terminal.draw(|f| ui::draw(f))?;

        match events.next()? {
            Event::Input(key) => match key {
                Key::Char('q') => {
                    dbg!("quit");
                    should_quit = true;
                }
                Key::Char(c) => {
                    dbg!(c);
                }
                _ => {}
            },
            Event::File(event) => {
                dbg!(event.path);
            }
            _ => {}
        }

        if should_quit {
            break;
        }
    }

    Ok(())
}
