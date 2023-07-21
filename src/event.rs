use std::{fmt::Display, time::Duration};

use rdev::{listen, simulate, Button, Event as RdEvent, EventType};

use crate::{keys::Key, mouse::MouseButton};

#[derive(Debug, Clone)]
pub enum RawEvent {
    Terminate,
    Event(RdEvent),
}

#[derive(Debug, Clone)]
pub struct Event {
    pub delay: Duration,
    pub event: EventType,
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
