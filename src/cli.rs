use crate::app_state::{AppStateManager, Event};
use crate::Program;
use async_trait::async_trait;

pub struct CliApp {}

#[async_trait]
impl Program for CliApp {
    async fn run<'stream>(&self, app: AppStateManager) -> anyhow::Result<()> {
        let watch_state = app.state();
        tokio::pin!(watch_state);

        loop {
            let (event, app_state) = watch_state.borrow().clone();

            match event {
                Event::FilesChanged(files) => {
                    println!("{:?}", files);
                    println!("So I reckon the following have now changed:");
                    println!("{:?}", app_state.watched_files);
                }
                _ => {}
            }

            if watch_state.changed().await.is_err() {
                break;
            }
        }

        Ok(())
    }
}
