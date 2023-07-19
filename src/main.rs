#![allow(unused)]

use rdev::{listen, simulate, Button, Event as RdEvent, EventType};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::sync::mpsc::channel;
use std::time::{Duration, SystemTime};
use std::{env, thread};

use crate::keys::{Key, KeyState};
use crate::mouse::MouseButton;
mod keys;
mod mouse;

const FILE: &str = "result";

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
                write!(f, "{},kp,{}", self.delay.as_millis(), Key::from(key) as u8)
            }
            EventType::KeyRelease(key) => {
                write!(f, "{},kr,{}", self.delay.as_millis(), Key::from(key) as u8)
            }
            EventType::ButtonPress(button) => {
                write!(
                    f,
                    "{},mp,{}",
                    self.delay.as_millis(),
                    MouseButton::from(button) as u8
                )
            }
            EventType::ButtonRelease(button) => {
                write!(
                    f,
                    "{},mr,{}",
                    self.delay.as_millis(),
                    MouseButton::from(button) as u8
                )
            }
            EventType::MouseMove { x, y } => {
                write!(f, "{},mm,{},{}", self.delay.as_millis(), x as i64, y as i64)
            }
            EventType::Wheel { delta_x, delta_y } => {
                write!(f, "{},mw,{},{}", self.delay.as_millis(), delta_x, delta_y)
            }
        }
    }
}

fn record() {
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

    let mut keystate: HashMap<rdev::Key, bool> = HashMap::with_capacity(110);

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
                }
            }
        };
    }

    let mut file = File::create(FILE).unwrap();
    for event in events {
        writeln!(file, "{}", event).unwrap();
    }

    println!("bye!");
}

fn playback() {
    let (tx, rx) = channel();
    let _listener = thread::spawn(move || {
        listen(move |event| {
            if let EventType::KeyPress(key) = event.event_type {
                if let rdev::Key::Space = key {
                    tx.send(RawEvent::Terminate)
                        .unwrap_or_else(|e| println!("Could not send event {:?}", e));
                    return;
                }
            }
        })
    });

    let contents = std::fs::read_to_string(FILE).unwrap();
    let mut events = Vec::new();
    for line in contents.lines() {
        let mut values = line.split(",");
        let delay_value = values
            .next()
            .map(|s| u64::from_str_radix(s, 10).unwrap())
            .unwrap();

        let delay = Duration::from_millis(delay_value);
        let event = match values.next().unwrap() {
            "kp" => {
                let key = Key::from(values.next().map(|s| s.parse::<u8>().unwrap()).unwrap());
                Event {
                    delay,
                    event: EventType::KeyPress(key.into()),
                }
            }
            "kr" => {
                let key = Key::from(values.next().map(|s| s.parse::<u8>().unwrap()).unwrap());
                Event {
                    delay,
                    event: EventType::KeyRelease(key.into()),
                }
            }
            "mp" => {
                let button =
                    MouseButton::from(values.next().map(|s| s.parse::<u8>().unwrap()).unwrap());
                Event {
                    delay,
                    event: EventType::ButtonPress(button.into()),
                }
            }
            "mr" => {
                let button =
                    MouseButton::from(values.next().map(|s| s.parse::<u8>().unwrap()).unwrap());
                Event {
                    delay,
                    event: EventType::ButtonRelease(button.into()),
                }
            }
            "mm" => {
                let x = values.next().map(|s| s.parse::<f64>().unwrap()).unwrap();
                let y = values.next().map(|s| s.parse::<f64>().unwrap()).unwrap();
                Event {
                    delay,
                    event: EventType::MouseMove { x, y },
                }
            }
            "mw" => {
                let delta_x = values.next().map(|s| s.parse::<i64>().unwrap()).unwrap();
                let delta_y = values.next().map(|s| s.parse::<i64>().unwrap()).unwrap();
                Event {
                    delay,
                    event: EventType::Wheel { delta_x, delta_y },
                }
            }
            _ => todo!(),
        };
        events.push(event);
    }

    for event in events {
        if let Ok(msg) = rx.try_recv() {
            break;
        }

        spin_sleep::sleep(event.delay);
        simulate(&event.event).expect(&format!("failed to simulate {:#?}", event));
    }
}

fn main() {
    let args = env::args();
    let command = args.skip(1).next().unwrap_or("rec".to_string());

    match command.as_str() {
        "play" => playback(),
        "rec" => record(),
        _ => println!("unknown command {}", command),
    };
}
