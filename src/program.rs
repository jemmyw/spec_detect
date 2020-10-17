use crate::AppStateManager;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Program {
    async fn run<'stream>(&self, app: AppStateManager) -> Result<()>;
}
