use std::path::PathBuf;

use clap::{crate_description, crate_version, Args, Parser, Subcommand};

pub trait Run {
    fn run(self) -> eyre::Result<()>;
}

#[derive(Debug, Parser)]
#[command(
    name = "mkrp",
    about = crate_description!(),
    version = crate_version!(),
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Cmd,
}

impl Run for Cli {
    fn run(self) -> eyre::Result<()> {
        match self.command {
            Cmd::Rec(c) => c.run(),
            Cmd::Play(c) => c.run(),
            Cmd::Interactive(c) => c.run(),
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    Rec(Record),
    Play(Play),
    Interactive(Interactive),
}

/// Record mouse and keyboard events and save them into a file
#[derive(Debug, Args)]
#[command(
    visible_alias("r"),
    disable_colored_help(true),
    disable_version_flag(true)
)]
pub struct Record {
    /// Key to be used to stop recording
    ///
    /// The stop key can be any combination of keys that have to be either pressed or held down at
    /// the same time. The key combination is a comma seperated list of keys.
    ///
    /// If no value is passed this is defaulted to `Escape` as the stop key.
    ///
    /// Example:
    ///     Stop the recording with the key combo `Ctrl` + `F9` would be `ctrl,f9`.
    #[arg(short, long)]
    pub stop_key: Option<String>,

    /// Output recorded events into path.
    #[arg(value_name = "PATH")]
    pub output: PathBuf,
}

/// Play recorded file
#[derive(Debug, Args)]
#[command(
    visible_alias("p"),
    disable_colored_help(true),
    disable_version_flag(true)
)]
pub struct Play {
    /// Number of iterations to be executed
    #[arg(short, long)]
    pub iterations: Option<u32>,

    /// Key to be used to stop playback
    ///
    /// The stop key can be any combination of keys that have to be either pressed or held down at
    /// the same time. The key combination is a comma seperated list of keys.  If no value is
    /// passed this is defaulted to `Escape` as the stop key.
    ///
    /// Example:
    ///     Stop the playback with the key combo `Ctrl` + `F9` would be `ctrl,f9`.
    #[arg(short, long)]
    pub stop_key: Option<String>,

    /// Delay between two iterations
    #[arg(short, long)]
    pub delay: Option<u64>,

    /// Input file to be played
    #[arg(value_name = "PATH")]
    pub output: PathBuf,
}

/// Interactive mode
#[derive(Debug, Args)]
#[command(
    visible_alias("i"),
    disable_colored_help(true),
    disable_version_flag(true)
)]
pub struct Interactive {
    /// Key that will terminate interactive mode
    #[arg(short, long)]
    pub terminate_key: Option<String>,

    /// Key that will stop either recording or replaying
    #[arg(short, long)]
    pub stop_key: Option<String>,

    /// Key that will pause recording or replaying actions
    #[arg(short, long)]
    pub pause_key: Option<String>,

    /// Key to start recording
    #[arg(short, long)]
    pub rec_key: Option<String>,

    /// Key to start replaying
    #[arg(short = 'y', long)]
    pub play_key: Option<String>,
}
