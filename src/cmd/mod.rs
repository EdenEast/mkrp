mod play;
mod record;

use async_trait::async_trait;

use crate::cli::{Cli, Cmd};

#[async_trait]
pub trait Run {
    async fn run(self) -> eyre::Result<()>;
}

pub async fn run(cli: Cli) -> eyre::Result<()> {
    match cli.command {
        Cmd::Rec(c) => c.run().await,
        Cmd::Play(c) => c.run().await,
    }
}
