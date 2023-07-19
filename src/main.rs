use rdev::{listen, Event as RdEvent, EventType, Key};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, SystemTime};
use str_ext::StrExt;

mod str_ext;

#[derive(Debug, Clone)]
enum RawEvent {
    Terminate,
    Event(RdEvent),
}

#[derive(Debug, Clone)]
struct Event {
    delay: Duration,
    event: EventType,
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.event {
            EventType::KeyPress(key) => {
                write!(f, "{},kp,{}", self.delay.as_millis(), key.ser_to_str())
            }
            EventType::KeyRelease(key) => {
                write!(f, "{},kr,{}", self.delay.as_millis(), key.ser_to_str())
            }
            EventType::ButtonPress(button) => {
                write!(f, "{},mp,{}", self.delay.as_millis(), button.ser_to_str())
            }
            EventType::ButtonRelease(button) => {
                write!(f, "{},mr,{}", self.delay.as_millis(), button.ser_to_str())
            }
            EventType::MouseMove { x, y } => {
                write!(f, "{},mm,{},{}", self.delay.as_millis(), x, y)
            }
            EventType::Wheel { delta_x, delta_y } => {
                write!(f, "{},mw,{},{}", self.delay.as_millis(), delta_x, delta_y)
            }
        }
    }
}

fn main() {
    // spawn new thread because listen blocks
    let (tx, rx) = channel();
    let mut prev_system_time = SystemTime::now();

    let _listener = thread::spawn(move || {
        listen(move |event| {
            match event.event_type {
                rdev::EventType::KeyPress(rdev::Key::Space) => tx
                    .send(RawEvent::Terminate)
                    .unwrap_or_else(|e| println!("Could not send event {:?}", e)),
                _ => tx
                    .send(RawEvent::Event(event))
                    .unwrap_or_else(|e| println!("Could not send event {:?}", e)),
            };
        })
        .expect("Could not listen");
    });

    let mut keystate: HashMap<Key, bool> = HashMap::with_capacity(110);

    let mut events = Vec::new();
    for raw_event in rx.iter() {
        match raw_event {
            RawEvent::Terminate => break,
            RawEvent::Event(event) => {
                match event.event_type {
                    EventType::KeyPress(key) => {
                        // TODO: Could this just be a bitfield?
                        let state = keystate.get(&key).map(|b| *b).unwrap_or(false);
                        if !state {
                            let duration = event
                                .time
                                .duration_since(prev_system_time)
                                .expect("failed to get duration since last event");

                            events.push(Event {
                                delay: duration,
                                event: event.event_type,
                            });

                            keystate.insert(key, true);
                            prev_system_time = event.time;
                            println!("{:?} Pressed, Duration: {:?}", key, duration);
                        }
                    }
                    EventType::KeyRelease(key) => {
                        let state = keystate.get(&key).map(|b| *b).unwrap_or(false);
                        if state {
                            let duration = event
                                .time
                                .duration_since(prev_system_time)
                                .expect("failed to get duration since last event");

                            events.push(Event {
                                delay: duration,
                                event: event.event_type,
                            });

                            keystate.insert(key, false);
                            prev_system_time = event.time;
                            println!("{:?} Released, Duration: {:?}", key, duration);
                        }
                    }
                    EventType::MouseMove { x, y } => {
                        let duration = event
                            .time
                            .duration_since(prev_system_time)
                            .expect("failed to get duration since last event");
                        if duration.as_millis() >= 10 {
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
                }
            }
        };
    }

    let mut file = File::create("result.json").unwrap();
    for event in events {
        writeln!(file, "{}", event).unwrap();
    }

    println!("bye!");
}
