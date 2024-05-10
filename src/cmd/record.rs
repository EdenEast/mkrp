use std::{
    fs::File,
    io::Write,
    path::is_separator,
    str::FromStr,
    sync::mpsc::channel,
    thread,
    time::{Duration, SystemTime},
};

use rdev::{listen, simulate};

use crate::{
    cli::{Record, Run},
    event::Event,
    keys::{Key, KeyState},
};

impl Run for Record {
    fn run(self) -> eyre::Result<()> {
        let stop_state = match self.stop_key {
            Some(s) => KeyState::parse_cli_str(&s)?,
            None => KeyState::with_pressed(&[Key::Escape]),
        };

        let pause_state = match self.pause_key {
            Some(p) => KeyState::parse_cli_str(&p)?,
            None => KeyState::with_pressed(&[Key::Insert]),
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
        let mut is_paused = false;
        let mut last_mouse_position: Option<(f64, f64)> = None;
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

                        if keystate.is_state_held(pause_state) {
                            is_paused = !is_paused;
                            if !is_paused {
                                prev_system_time = event.time;

                                // Reset mouse position if reset is set
                                if self.reset {
                                    if let Some(pos) = last_mouse_position {
                                        simulate(&rdev::EventType::MouseMove {
                                            x: pos.0,
                                            y: pos.1,
                                        })
                                        .unwrap_or_else(
                                            |_| {
                                                panic!(
                                                    "failed to move mouse back to position ({},{})",
                                                    pos.0 as i32, pos.1 as i32
                                                )
                                            },
                                        );
                                    }
                                    // Add small delay to make sure that os catchs up and applies
                                    // the mouse update
                                    spin_sleep::sleep(Duration::from_millis(50));
                                }
                                println!("Unpaused");
                            } else {
                                println!("Paused")
                            }
                            continue;
                        }

                        if is_paused {
                            continue;
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
                        keystate.set_released(key);
                        if is_paused {
                            continue;
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
                        println!("{:?} Released, Duration: {:?}", key, duration);
                    }
                }
                rdev::EventType::MouseMove { x, y } => {
                    if is_paused {
                        continue;
                    }

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
                        last_mouse_position = Some((x, y));
                        println!("Move ({},{}), Duration {:?}", x, y, duration);
                    }
                }
                e => {
                    if is_paused {
                        continue;
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
