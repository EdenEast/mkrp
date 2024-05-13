#[derive(Debug, Default, Clone, Copy)]
pub struct MouseState(u8);

impl MouseState {
    pub fn with_pressed(buttons: &[MouseButton]) -> Self {
        let mut state = Self::default();
        for k in buttons {
            state.set_pressed(*k);
        }
        state
    }

    pub fn is_pressed(&self, button: MouseButton) -> bool {
        (self.0 >> button as u8) & 1u8 == 1
    }

    pub fn set_pressed(&mut self, button: MouseButton) {
        self.0 |= 1 << (button as u8);
    }

    pub fn set_released(&mut self, button: MouseButton) {
        self.0 &= !(1 << (button as u8));
    }

    pub fn is_state_held(&self, state: MouseState) -> bool {
        self.0 & state.0 == state.0
    }

    pub fn iter(&self) -> MouseIterator {
        MouseIterator::new(*self)
    }
}

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

pub struct MouseIterator {
    value: u8,
}

impl MouseIterator {
    pub fn new(value: MouseState) -> Self {
        Self { value: value.0 }
    }
}

impl Iterator for MouseIterator {
    type Item = MouseButton;

    fn next(&mut self) -> Option<Self::Item> {
        if self.value == 0 {
            return None;
        }

        let index = self.value.trailing_zeros() as u8;
        self.value &= !(1 << index);
        Some(index.into())
    }
}

impl IntoIterator for MouseState {
    type Item = MouseButton;
    type IntoIter = MouseIterator;

    fn into_iter(self) -> Self::IntoIter {
        MouseIterator::new(self)
    }
}
