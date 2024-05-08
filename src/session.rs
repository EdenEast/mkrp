use std::{fs::read_to_string, path::Path, time::Duration};

use rdev::EventType;

use crate::{event::Event, keys::Key, mouse::MouseButton};

pub struct Session {
    pub events: Vec<Event>,
    pub total_time: Duration,
}

impl Session {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let contents = std::fs::read_to_string(path.as_ref())?;
        Ok(Self::from_str(&contents))
    }

    pub fn from_str(contents: &str) -> Self {
        let mut total_time = Duration::ZERO;
        let mut events = Vec::new();
        for line in contents.lines() {
            let mut values = line.split(',');
            let delay_value = values.next().map(|s| s.parse().unwrap()).unwrap();

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
            total_time += delay;
        }

        Self { events, total_time }
    }
}
