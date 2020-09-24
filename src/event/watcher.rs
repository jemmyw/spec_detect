use notify::{raw_watcher, PollWatcher, RecommendedWatcher, RecursiveMode};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

/// Thin wrapper over the notify crate
///
/// `PollWatcher` and `RecommendedWatcher` are distinct types, but watchexec
/// really just wants to handle them without regard to the exact type
/// (e.g. polymorphically). This has the nice side effect of separating out
/// all coupling to the notify crate into this module.
pub struct Watcher {
    watcher_impl: WatcherImpl,
    thread: thread::JoinHandle<()>,
}

pub use notify::Error;
pub use notify::RawEvent;

pub struct Event {
    pub path: Option<PathBuf>,
}

enum WatcherImpl {
    Recommended(RecommendedWatcher),
    Poll(PollWatcher),
}

impl Watcher {
    pub fn new(
        tx: Sender<Event>,
        paths: &[PathBuf],
        poll: bool,
        interval_ms: u32,
    ) -> Result<Self, Error> {
        use notify::Watcher;
        let (raw_tx, raw_rx): (Sender<RawEvent>, Receiver<RawEvent>) = std::sync::mpsc::channel();

        let imp = if poll {
            let mut watcher = PollWatcher::with_delay_ms(raw_tx, interval_ms)?;
            for path in paths {
                watcher.watch(path, RecursiveMode::Recursive)?;
                dbg!("Watching {:?}", path);
            }

            WatcherImpl::Poll(watcher)
        } else {
            let mut watcher = raw_watcher(raw_tx)?;
            for path in paths {
                watcher.watch(path, RecursiveMode::Recursive)?;
                dbg!("Watching {:?}", path);
            }

            WatcherImpl::Recommended(watcher)
        };

        let tx = tx.clone();
        let thread = thread::spawn(move || loop {
            let raw_event = raw_rx.recv();

            match raw_event {
                Ok(raw_event) => {
                    let path = raw_event.path;
                    dbg!(path.clone());
                    let event = Event { path };
                    tx.send(event);
                }
                _ => {}
            }
        });

        Ok(Self {
            watcher_impl: imp,
            thread,
        })
    }

    pub fn is_polling(&self) -> bool {
        if let WatcherImpl::Poll(_) = self.watcher_impl {
            true
        } else {
            false
        }
    }
}
