use std::{sync::mpsc::channel, thread, time::Duration};

use rdev::{listen, simulate, EventType};

use crate::{
    cli::{Play, Run},
    event::RawEvent,
    keys::{Key, KeyState},
    session::Session,
};

impl Run for Play {
    fn run(self) -> eyre::Result<()> {
        let session = Session::from_file(self.output).unwrap();

        let stop_state = match self.stop_key {
            Some(s) => {
                let mut state = KeyState::default();
                for item in s.split(",") {
                    let key = Key::from_str(item)
                        .ok_or(eyre::eyre!("Unknown key '{}' for stop key", item))?;
                    state.set_pressed(key);
                }
                state
            }
            None => KeyState::with_pressed(&[Key::Escape]),
        };

        let delay = self.delay.map(|s| Duration::from_millis(s));
        let total_iterations = self.iterations.unwrap_or(1);

        let (tx, rx) = channel();
        let _listener = thread::spawn(move || {
            let mut keystate = KeyState::default();
            listen(move |event| match event.event_type {
                EventType::KeyPress(k) => {
                    keystate.set_pressed(k.into());
                    if keystate.is_state_held(stop_state) {
                        tx.send(true)
                            .unwrap_or_else(|_| println!("Could not send terminate event"));
                    }
                }
                EventType::KeyRelease(k) => {
                    keystate.set_released(k.into());
                }
                _ => {}
            })
            .expect("Could not listen");
        });

        let mut has_terminated = false;
        for i in 0..total_iterations {
            for event in &session.events {
                if let Ok(msg) = rx.try_recv() {
                    has_terminated = true;
                    break;
                }
                spin_sleep::sleep(event.delay);
                simulate(&event.event).expect(&format!("failed to simulate {:#?}", event));
            }

            if has_terminated {
                break;
            }

            if let Some(delay) = delay {
                spin_sleep::sleep(delay);
            }
        }

        Ok(())
    }
}
