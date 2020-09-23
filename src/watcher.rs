use notify::{EventFn, RecommendedWatcher, RecursiveMode, Result, Watcher};

pub struct FilesWatcher {
    watcher: RecommendedWatcher,
}

impl FilesWatcher {
    fn new(event_fn: EventFn) -> Result<Self> {
        let mut watcher: RecommendedWatcher = Watcher::new_immediate(|res| match res {
            Ok(event) => println!("event: {:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        })?;

        FilesWatcher {}
    }
}

pub fn watch() -> Result<()> {
    // Automatically select the best implementation for your platform.
    let mut watcher: RecommendedWatcher = Watcher::new_immediate(|res| match res {
        Ok(event) => println!("event: {:?}", event),
        Err(e) => println!("watch error: {:?}", e),
    })?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(".", RecursiveMode::Recursive)?;

    Ok(())
}
