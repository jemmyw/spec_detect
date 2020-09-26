mod app;
mod code_repo;
mod event;
mod ui;

use app::App;
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

    let repo = match CodeRepo::open(".") {
        Ok(r) => r,
        Err(_) => {
            panic!();
        }
    };

    let mut config = Config::default();
    config.paths = vec!["src".to_owned()];
    let events = Events::with_config(config);
    let mut app = App::new(repo);

    terminal.clear()?;

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

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
            Event::File(event) => app.on_file_event(event),
            _ => {}
        }

        if app.should_quit {
            break;
        }
    }

    terminal.clear()?;

    Ok(())
}
