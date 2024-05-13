use crate::cli::{Cli, Run};

mod play;
mod record;

pub async fn run(cli: Cli) -> eyre::Result<()> {
    match cli.command {
        crate::cli::Cmd::Record(c) => c.run().await,
        crate::cli::Cmd::Play(c) => c.run().await,
    }
}
