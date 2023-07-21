use std::{sync::mpsc::channel, thread};

use eyre::Result;
use rdev::listen;

use crate::{
    cli::{Interactive, Run},
    keys::{Key, KeyState},
};

fn keystate_from_arg(arg: Option<&str>, default: &[Key]) -> Result<KeyState> {
    match arg {
        Some(s) => {
            let mut state = KeyState::default();
            for item in s.split(",") {
                let key = Key::from_str(item)
                    .ok_or(eyre::eyre!("Unknown key '{}', in key sequence", item))?;
                state.set_pressed(key);
            }
            Ok(state)
        }
        None => Ok(KeyState::with_pressed(default)),
    }
}

enum Mode {
    Rec,
    Playback,
    Idle,
}

pub struct Keystates {
    pub keystate: KeyState,
    term: KeyState,
    stop: KeyState,
    rec: KeyState,
    play: KeyState,
    pause: KeyState,
}

impl Keystates {
    pub fn from_args(args: &Interactive) -> Result<Self> {
        Ok(Self {
            keystate: KeyState::default(),
            term: keystate_from_arg(args.terminate_key.as_deref(), &[Key::Escape])?,
            stop: keystate_from_arg(args.stop_key.as_deref(), &[Key::F12])?,
            rec: keystate_from_arg(args.rec_key.as_deref(), &[Key::F9])?,
            play: keystate_from_arg(args.play_key.as_deref(), &[Key::F10])?,
            pause: keystate_from_arg(args.pause_key.as_deref(), &[Key::F11])?,
        })
    }
}

impl Run for Interactive {
    fn run(self) -> eyre::Result<()> {
        let (tx, rx) = channel();
        let _listener = thread::spawn(move || {
            listen(move |event| {
                tx.send(event)
                    .unwrap_or_else(|e| println!("Could not send event {:?}", e));
            })
            .expect("Could not listen");
        });

        let events = Vec::new();
        let mode = Mode::Idle;
        let states = Keystates::from_args(&self)?;
        for event in rx.iter() {
            match mode {
                Mode::Rec => match event.event_type {
                    rdev::EventType::KeyPress(k) => todo!(),
                    rdev::EventType::KeyRelease(k) => todo!(),
                    rdev::EventType::ButtonPress(_) => todo!(),
                    rdev::EventType::ButtonRelease(_) => todo!(),
                    rdev::EventType::MouseMove { x, y } => todo!(),
                    rdev::EventType::Wheel { delta_x, delta_y } => todo!(),
                },
                Mode::Playback => todo!(),
                Mode::Idle => todo!(),
            }
        }

        Ok(())
    }
}
