#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum MouseButton {
    Left = 1,
    Right,
    Middle,
    Unknown,
}

impl From<u8> for MouseButton {
    fn from(value: u8) -> Self {
        match value {
            1 => MouseButton::Left,
            2 => MouseButton::Right,
            3 => MouseButton::Middle,
            _ => MouseButton::Unknown,
        }
    }
}

impl From<MouseButton> for u8 {
    fn from(value: MouseButton) -> Self {
        value as u8
    }
}

impl From<rdev::Button> for MouseButton {
    fn from(value: rdev::Button) -> Self {
        match value {
            rdev::Button::Left => MouseButton::Left,
            rdev::Button::Right => MouseButton::Right,
            rdev::Button::Middle => MouseButton::Middle,
            rdev::Button::Unknown(_) => MouseButton::Unknown,
        }
    }
}

impl From<MouseButton> for rdev::Button {
    fn from(value: MouseButton) -> Self {
        match value {
            MouseButton::Left => rdev::Button::Left,
            MouseButton::Right => rdev::Button::Right,
            MouseButton::Middle => rdev::Button::Middle,
            MouseButton::Unknown => rdev::Button::Unknown(0),
        }
    }
}
