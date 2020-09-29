use notify::{watcher, RecommendedWatcher, RecursiveMode};
use std::path::PathBuf;
use std::sync::mpsc::Sender;

pub struct Watcher {
    _watcher_impl: WatcherImpl,
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

        Ok(Self { _watcher_impl: imp })
    }
}
