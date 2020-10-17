use crate::AppStateManager;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Program {
    async fn run<'stream>(&self, app: AppStateManager) -> Result<()>;
}

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt()]
pub struct Opt {
    #[structopt(long)]
    pub cli: bool,
    #[structopt(long)]
    pub ui: bool,
}
