use std::io;
use std::thread;

use tokio::sync::broadcast::{self, Receiver};

use termion::event::Key;
use termion::input::TermRead;

pub fn listen() -> Receiver<Key> {
    let (tx, rx) = broadcast::channel(1);
    tx.send(Key::Null).unwrap();

    let _input_handle = {
        thread::spawn(move || {
            let stdin = io::stdin();
            for evt in stdin.keys() {
                if let Ok(key) = evt {
                    if let Err(err) = tx.send(key) {
                        eprintln!("{}", err);
                        return;
                    }
                }
            }
        })
    };

    rx
}
