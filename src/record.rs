use std::{fs::File, io::Write, sync::mpsc::channel, thread, time::SystemTime};

use rdev::listen;

use crate::{
    cli::{Record, Run},
    event::{Event, RawEvent},
    keys::{Key, KeyState},
};

impl Run for Record {
    fn run(self) -> eyre::Result<()> {
        // spawn new thread because listen blocks
        let (tx, rx) = channel();
        let mut prev_system_time = SystemTime::now();

        let _listener = thread::spawn(move || {
            listen(move |event| {
                match event.event_type {
                    rdev::EventType::KeyPress(rdev::Key::F9) => tx
                        .send(RawEvent::Terminate)
                        .unwrap_or_else(|e| println!("Could not send event {:?}", e)),
                    _ => tx
                        .send(RawEvent::Event(event))
                        .unwrap_or_else(|e| println!("Could not send event {:?}", e)),
                };
            })
            .expect("Could not listen");
        });

        let mut keystate = KeyState::default();
        let mut events = Vec::new();
        for raw_event in rx.iter() {
            match raw_event {
                RawEvent::Terminate => break,
                RawEvent::Event(event) => match event.event_type {
                    rdev::EventType::KeyPress(rkey) => {
                        let key: Key = rkey.into();
                        if !keystate.is_pressed(key) {
                            let duration = event
                                .time
                                .duration_since(prev_system_time)
                                .expect("failed to get duration since last event");

                            events.push(Event {
                                delay: duration,
                                event: event.event_type,
                            });
                            keystate.set_pressed(key);
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
                },
            }
        }

        let mut file = File::create(self.output).unwrap();
        for event in events {
            writeln!(file, "{}", event).unwrap();
        }
        println!("bye!");

        Ok(())
    }
}
