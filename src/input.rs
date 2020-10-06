use std::io;
use std::thread;

use tokio::sync::watch::{self, Receiver};

use termion::event::Key;
use termion::input::TermRead;

pub fn listen() -> Receiver<Key> {
    let (tx, rx) = watch::channel(Key::Null);
    let _input_handle = {
        thread::spawn(move || {
            let stdin = io::stdin();
            for evt in stdin.keys() {
                if let Ok(key) = evt {
                    if let Err(err) = tx.broadcast(key) {
                        eprintln!("{}", err);
                        return;
                    }
                }
            }
        })
    };

    rx
}
