use std::io;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use termion::event::Key;
use termion::input::TermRead;

pub mod watcher;
pub use watcher::DebouncedEvent;
use watcher::Watcher;

pub enum Event<I, F> {
    Input(I),
    Tick,
    File(F),
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: mpsc::Receiver<Event<Key, watcher::DebouncedEvent>>,
    _input_handle: thread::JoinHandle<()>,
    _tick_handle: thread::JoinHandle<()>,
    _watcher_handle: (Watcher, thread::JoinHandle<()>),
}

impl Events {
    pub fn new() -> anyhow::Result<Events> {
        let tick_rate = Duration::from_millis(1000);

        let (tx, rx) = mpsc::channel();
        let _input_handle = {
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

        let _tick_handle = {
            let tx = tx.clone();
            thread::spawn(move || loop {
                if let Err(_) = tx.send(Event::Tick) {
                    return;
                }
                thread::sleep(tick_rate);
            })
        };

        let _watcher_handle = {
            let tx = tx.clone();
            let (w_tx, w_rx) = mpsc::channel();
            let watcher = Watcher::new(w_tx, &[PathBuf::from(".")], 0).unwrap();
            let thread_handle = thread::spawn(move || loop {
                match w_rx.recv() {
                    Ok(event) => tx.send(Event::File(event)).expect("could not send event"),
                    _ => {}
                }
            });

            (watcher, thread_handle)
        };

        Ok(Events {
            rx,
            _input_handle,
            _tick_handle,
            _watcher_handle,
        })
    }

    pub fn next(&self) -> Result<Event<Key, watcher::DebouncedEvent>, mpsc::RecvError> {
        self.rx.recv()
    }
}
