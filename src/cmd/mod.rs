use crate::cli::{Cli, Run};

mod play;
mod record;

pub fn run(cli: Cli) -> eyre::Result<()> {
    match cli.command {
        crate::cli::Cmd::Rec(c) => c.run(),
        crate::cli::Cmd::Play(c) => c.run(),
    }
}
