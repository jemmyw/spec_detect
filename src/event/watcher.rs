use notify::{watcher, RecommendedWatcher, RecursiveMode};
use std::path::PathBuf;
use std::sync::mpsc::Sender;

/// Thin wrapper over the notify crate
///
/// `PollWatcher` and `RecommendedWatcher` are distinct types, but watchexec
/// really just wants to handle them without regard to the exact type
/// (e.g. polymorphically). This has the nice side effect of separating out
/// all coupling to the notify crate into this module.
pub struct Watcher {
    watcher_impl: WatcherImpl,
}

pub use notify::DebouncedEvent;
pub use notify::Error;

enum WatcherImpl {
    Recommended(RecommendedWatcher),
}

impl Watcher {
    pub fn new(
        tx: Sender<DebouncedEvent>,
        paths: &[PathBuf],
        interval_ms: u64,
    ) -> Result<Self, Error> {
        use notify::Watcher;

        let imp = {
            let mut watcher = watcher(tx, std::time::Duration::from_millis(interval_ms))?;
            for path in paths {
                watcher.watch(path, RecursiveMode::Recursive)?;
                dbg!("INotify Watching {:?}", path);
            }

            WatcherImpl::Recommended(watcher)
        };

        Ok(Self { watcher_impl: imp })
    }
}
