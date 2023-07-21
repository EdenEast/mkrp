#![allow(unused)]

use clap::Parser;
use cli::{Play, Run};
use rdev::{listen, simulate, Button, Event as RdEvent, EventType};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::time::{Duration, SystemTime};
use std::{env, thread};

use crate::keys::{Key, KeyState};
use crate::mouse::MouseButton;

mod cli;
mod event;
mod keys;
mod mouse;
mod play;
mod record;
mod session;

const FILE: &str = "rec.mkrp";

fn main() {
    let command = cli::Cli::parse();
    match command.command {
        cli::Cmd::Rec(c) => c.run().unwrap(),
        cli::Cmd::Play(c) => c.run().unwrap(),
    }
}
