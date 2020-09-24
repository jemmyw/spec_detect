use std::io;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{
    atomic::{AtomicBool, AtomicI32, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use termion::event::Key;
use termion::input::TermRead;

mod watcher;
use watcher::Watcher;

pub enum Event<I, F> {
    Input(I),
    Tick,
    File(F),
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event<Key, watcher::Event>>,
    input_handle: thread::JoinHandle<()>,
    tick_handle: thread::JoinHandle<()>,
    watcher_handle: thread::JoinHandle<()>,
}

#[derive(Debug)]
pub struct Config {
    pub exit_key: Key,
    pub tick_rate: Duration,
    pub paths: Vec<String>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            exit_key: Key::Char('q'),
            tick_rate: Duration::from_millis(1000),
            paths: vec![String::from(".")],
        }
    }
}

impl Clone for Config {
    fn clone(&self) -> Config {
        Config {
            exit_key: self.exit_key.clone(),
            tick_rate: self.tick_rate.clone(),
            paths: self.paths.clone(),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    if let Ok(key) = evt {
                        if let Err(err) = tx.send(Event::Input(key)) {
                            eprintln!("{}", err);
                            return;
                        }
                    }
                }
            })
        };
        let tick_rate = config.tick_rate.to_owned();
        let tick_handle = {
            let tx = tx.clone();
            thread::spawn(move || loop {
                tx.send(Event::Tick).unwrap();
                thread::sleep(tick_rate);
            })
        };

        let watcher_handle = {
            let paths = config.paths;
            let paths_iter = paths.iter().map(|p| PathBuf::from(p));
            let paths_vec = paths_iter.collect::<Vec<PathBuf>>();

            let tx = tx.clone();

            thread::spawn(move || {
                let (w_tx, w_rx) = mpsc::channel();
                Watcher::new(w_tx, paths_vec.as_slice(), false, 0).unwrap();

                loop {
                    match w_rx.recv() {
                        Ok(event) => tx.send(Event::File(event)).expect("could not send event"),
                        _ => {}
                    }
                }
            })
        };

        Events {
            rx,
            input_handle,
            tick_handle,
            watcher_handle,
        }
    }

    pub fn next(&self) -> Result<Event<Key, watcher::Event>, mpsc::RecvError> {
        self.rx.recv()
    }
}
