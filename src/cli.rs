use crate::app_state::{AppStateManager, Event};
use crate::some_loop;
use crate::Program;
use async_trait::async_trait;
use tokio::stream::StreamExt;

pub struct CliApp {}

#[async_trait]
impl Program for CliApp {
    async fn run<'stream>(&self, app: AppStateManager) -> anyhow::Result<()> {
        let watch_state = app.stream();
        tokio::pin!(watch_state);

        some_loop!((event, app_state) = watch_state.next() => {
            match event {
                Event::FilesChanged(files) => {
                    println!("{:?}", files);
                    println!("So I reckon the following have now changed:");
                    println!("{:?}", app_state.changed_files);
                },
                _ => {}
            }
        });

        Ok(())
    }
}
