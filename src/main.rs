#![allow(unused)]

use clap::Parser;

mod cli;
mod cmd;
mod event;
mod keys;
mod mouse;
mod session;

fn main() -> eyre::Result<()> {
    cmd::run(cli::Cli::parse())
}
