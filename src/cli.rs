use std::path::PathBuf;

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

#[derive(Debug, Subcommand)]
pub enum Cmd {
    Rec(Record),
    Play(Play),
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
    #[arg(short, long, default_value_t = None)]
    pub stop_key: Option<String>,

    /// Output recorded events into path.
    #[arg(value_name = "PATH")]
    pub output: PathBuf,
}
