use std::{fs::File, io::Write, str::FromStr, sync::mpsc::channel, thread, time::SystemTime};

use rdev::listen;

use crate::{
    cli::{Record, Run},
    event::Event,
    keys::{Key, KeyState},
};

impl Run for Record {
    async fn run(self) -> eyre::Result<()> {
        let stop_state = match self.stop_key {
            Some(s) => {
                let mut state = KeyState::default();
                for item in s.split(',') {
                    let key = Key::from_str(item)
                        .ok_or(eyre::eyre!("Unknown key '{}' for stop key", item))?;
                    state.set_pressed(key);
                }
                state
            }
            None => KeyState::with_pressed(&[Key::Escape]),
        };

        // spawn new thread because listen blocks
        let (tx, rx) = channel();
        let mut prev_system_time = SystemTime::now();

        let _listener = thread::spawn(move || {
            listen(move |event| {
                tx.send(event)
                    .unwrap_or_else(|e| println!("Could not send event {:?}", e));
            })
            .expect("Could not listen");
        });

        let mut keystate = KeyState::default();
        let mut events = Vec::new();
        for event in rx.iter() {
            match event.event_type {
                rdev::EventType::KeyPress(rkey) => {
                    let key: Key = rkey.into();
                    if !keystate.is_pressed(key) {
                        // NOTE: Check if pressing this key satifies the stop state. This has to be
                        // checked first because we dont want to record the event as iterations
                        // would stop after the first iteration.
                        keystate.set_pressed(key);
                        if keystate.is_state_held(stop_state) {
                            break;
                        }

                        let duration = event
                            .time
                            .duration_since(prev_system_time)
                            .expect("failed to get duration since last event");

                        events.push(Event {
                            delay: duration,
                            event: event.event_type,
                        });
                        prev_system_time = event.time;
                        println!("{:?} Pressed, Duration: {:?}", key, duration);
                    }
                }
                rdev::EventType::KeyRelease(rkey) => {
                    let key: Key = rkey.into();
                    if keystate.is_pressed(key) {
                        let duration = event
                            .time
                            .duration_since(prev_system_time)
                            .expect("failed to get duration since last event");

                        events.push(Event {
                            delay: duration,
                            event: event.event_type,
                        });
                        keystate.set_released(key);
                        prev_system_time = event.time;
                        println!("{:?} Released, Duration: {:?}", key, duration);
                    }
                }
                rdev::EventType::MouseMove { x, y } => {
                    let duration = event
                        .time
                        .duration_since(prev_system_time)
                        .expect("failed to get duration since last event");
                    if duration.as_millis() >= 1 {
                        events.push(Event {
                            delay: duration,
                            event: event.event_type,
                        });

                        prev_system_time = event.time;
                        println!("Move ({},{}), Duration {:?}", x, y, duration);
                    }
                }
                e => {
                    let duration = event
                        .time
                        .duration_since(prev_system_time)
                        .expect("failed to get duration since last event");

                    events.push(Event {
                        delay: duration,
                        event: event.event_type,
                    });

                    prev_system_time = event.time;
                    println!("Received {:?}, Duration {:?}", e, duration);
                }
            };
        }

        let mut file = File::create(self.output).unwrap();
        for event in events {
            writeln!(file, "{}", event).unwrap();
        }
        println!("bye!");

        Ok(())
    }
}
