use std::{str::FromStr, string::ParseError};

#[derive(Debug, Default, Clone, Copy)]
pub struct KeyState(u128);

impl KeyState {
    pub fn is_pressed(&self, key: Key) -> bool {
        (self.0 >> key as u8) as u8 & 1u8 == 1
    }

    pub fn set_pressed(&mut self, key: Key) {
        self.0 |= 1 << (key as u8);
    }

    pub fn set_released(&mut self, key: Key) {
        self.0 &= !(1 << (key as u8));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Key {
    A = 1,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Num0,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    SemiColon,
    Comma,
    Dot,
    Slash,
    Backslash,
    LBracket,
    RBracket,
    Quote,
    Backquote,
    Minus,
    Equal,

    LCtrl,
    RCtrl,
    LShift,
    RShift,
    LAlt,
    RAlt,
    LSuper,
    RSuper,

    Backspace,
    Delete,
    Space,
    Return,
    Escape,
    Tab,

    Up,
    Down,
    Left,
    Right,

    Home,
    End,
    Pageup,
    Pagedown,

    Insert,
    PrintScreen,

    Unknown,
}

impl From<u8> for Key {
    fn from(value: u8) -> Self {
        if value == 0 || value >= Key::Unknown as u8 {
            return Key::Unknown;
        }

        /// SAFTY: The bounds of the value have been checked above, the rest of the values are
        /// valid keys.
        unsafe {
            std::mem::transmute(value)
        }
    }
}

impl From<Key> for u8 {
    fn from(value: Key) -> Self {
        value as u8
    }
}

impl From<Key> for rdev::Key {
    fn from(value: Key) -> Self {
        match value {
            Key::LAlt => rdev::Key::Alt,
            Key::RAlt => rdev::Key::AltGr,
            Key::Backspace => rdev::Key::Backspace,
            Key::LCtrl => rdev::Key::ControlLeft,
            Key::RCtrl => rdev::Key::ControlRight,
            Key::Delete => rdev::Key::Delete,
            Key::Down => rdev::Key::DownArrow,
            Key::End => rdev::Key::End,
            Key::Escape => rdev::Key::Escape,
            Key::F1 => rdev::Key::F1,
            Key::F10 => rdev::Key::F10,
            Key::F11 => rdev::Key::F11,
            Key::F12 => rdev::Key::F12,
            Key::F2 => rdev::Key::F2,
            Key::F3 => rdev::Key::F3,
            Key::F4 => rdev::Key::F4,
            Key::F5 => rdev::Key::F5,
            Key::F6 => rdev::Key::F6,
            Key::F7 => rdev::Key::F7,
            Key::F8 => rdev::Key::F8,
            Key::F9 => rdev::Key::F9,
            Key::Home => rdev::Key::Home,
            Key::Left => rdev::Key::LeftArrow,
            Key::Down => rdev::Key::PageDown,
            Key::Up => rdev::Key::PageUp,
            Key::Return => rdev::Key::Return,
            Key::Right => rdev::Key::RightArrow,
            Key::LShift => rdev::Key::ShiftLeft,
            Key::RShift => rdev::Key::ShiftRight,
            Key::Space => rdev::Key::Space,
            Key::Tab => rdev::Key::Tab,
            Key::Up => rdev::Key::UpArrow,
            Key::PrintScreen => rdev::Key::PrintScreen,
            Key::Backquote => rdev::Key::BackQuote,
            Key::Num1 => rdev::Key::Num1,
            Key::Num2 => rdev::Key::Num2,
            Key::Num3 => rdev::Key::Num3,
            Key::Num4 => rdev::Key::Num4,
            Key::Num5 => rdev::Key::Num5,
            Key::Num6 => rdev::Key::Num6,
            Key::Num7 => rdev::Key::Num7,
            Key::Num8 => rdev::Key::Num8,
            Key::Num9 => rdev::Key::Num9,
            Key::Num0 => rdev::Key::Num0,
            Key::Minus => rdev::Key::Minus,
            Key::Equal => rdev::Key::Equal,
            Key::Q => rdev::Key::KeyQ,
            Key::W => rdev::Key::KeyW,
            Key::E => rdev::Key::KeyE,
            Key::R => rdev::Key::KeyR,
            Key::T => rdev::Key::KeyT,
            Key::Y => rdev::Key::KeyY,
            Key::U => rdev::Key::KeyU,
            Key::I => rdev::Key::KeyI,
            Key::O => rdev::Key::KeyO,
            Key::P => rdev::Key::KeyP,
            Key::LBracket => rdev::Key::LeftBracket,
            Key::RBracket => rdev::Key::RightBracket,
            Key::A => rdev::Key::KeyA,
            Key::S => rdev::Key::KeyS,
            Key::D => rdev::Key::KeyD,
            Key::F => rdev::Key::KeyF,
            Key::G => rdev::Key::KeyG,
            Key::H => rdev::Key::KeyH,
            Key::J => rdev::Key::KeyJ,
            Key::K => rdev::Key::KeyK,
            Key::L => rdev::Key::KeyL,
            Key::SemiColon => rdev::Key::SemiColon,
            Key::Quote => rdev::Key::Quote,
            Key::Backslash => rdev::Key::BackSlash,
            Key::Z => rdev::Key::KeyZ,
            Key::X => rdev::Key::KeyX,
            Key::C => rdev::Key::KeyC,
            Key::V => rdev::Key::KeyV,
            Key::B => rdev::Key::KeyB,
            Key::N => rdev::Key::KeyN,
            Key::M => rdev::Key::KeyM,
            Key::Comma => rdev::Key::Comma,
            Key::Dot => rdev::Key::Dot,
            Key::Slash => rdev::Key::Slash,
            Key::Insert => rdev::Key::Insert,
            Key::Unknown => rdev::Key::Unknown(0),
            Key::LSuper => rdev::Key::MetaLeft,
            Key::RSuper => rdev::Key::MetaRight,
            Key::Pageup => rdev::Key::PageUp,
            Key::Pagedown => rdev::Key::PageDown,
        }
    }
}

impl From<rdev::Key> for Key {
    fn from(value: rdev::Key) -> Self {
        match value {
            rdev::Key::Alt => Key::LAlt,
            rdev::Key::AltGr => Key::RAlt,
            rdev::Key::Backspace => Key::Backspace,
            rdev::Key::CapsLock => todo!(),
            rdev::Key::ControlLeft => Key::LCtrl,
            rdev::Key::ControlRight => Key::RCtrl,
            rdev::Key::Delete => Key::Delete,
            rdev::Key::DownArrow => Key::Down,
            rdev::Key::End => Key::End,
            rdev::Key::Escape => Key::Escape,
            rdev::Key::F1 => Key::F1,
            rdev::Key::F10 => Key::F10,
            rdev::Key::F11 => Key::F11,
            rdev::Key::F12 => Key::F12,
            rdev::Key::F2 => Key::F2,
            rdev::Key::F3 => Key::F3,
            rdev::Key::F4 => Key::F4,
            rdev::Key::F5 => Key::F5,
            rdev::Key::F6 => Key::F6,
            rdev::Key::F7 => Key::F7,
            rdev::Key::F8 => Key::F8,
            rdev::Key::F9 => Key::F9,
            rdev::Key::Home => Key::Home,
            rdev::Key::LeftArrow => Key::Left,
            rdev::Key::MetaLeft => Key::LSuper,
            rdev::Key::MetaRight => Key::RSuper,
            rdev::Key::PageDown => Key::Down,
            rdev::Key::PageUp => Key::Up,
            rdev::Key::Return => Key::Return,
            rdev::Key::RightArrow => Key::Right,
            rdev::Key::ShiftLeft => Key::LShift,
            rdev::Key::ShiftRight => Key::RShift,
            rdev::Key::Space => Key::Space,
            rdev::Key::Tab => Key::Tab,
            rdev::Key::UpArrow => Key::Up,
            rdev::Key::PrintScreen => Key::PrintScreen,
            rdev::Key::ScrollLock => todo!(),
            rdev::Key::Pause => todo!(),
            rdev::Key::NumLock => todo!(),
            rdev::Key::BackQuote => Key::Backquote,
            rdev::Key::Num1 => Key::Num1,
            rdev::Key::Num2 => Key::Num2,
            rdev::Key::Num3 => Key::Num3,
            rdev::Key::Num4 => Key::Num4,
            rdev::Key::Num5 => Key::Num5,
            rdev::Key::Num6 => Key::Num6,
            rdev::Key::Num7 => Key::Num7,
            rdev::Key::Num8 => Key::Num8,
            rdev::Key::Num9 => Key::Num9,
            rdev::Key::Num0 => Key::Num0,
            rdev::Key::Minus => Key::Minus,
            rdev::Key::Equal => Key::Equal,
            rdev::Key::KeyQ => Key::Q,
            rdev::Key::KeyW => Key::W,
            rdev::Key::KeyE => Key::E,
            rdev::Key::KeyR => Key::R,
            rdev::Key::KeyT => Key::T,
            rdev::Key::KeyY => Key::Y,
            rdev::Key::KeyU => Key::U,
            rdev::Key::KeyI => Key::I,
            rdev::Key::KeyO => Key::O,
            rdev::Key::KeyP => Key::P,
            rdev::Key::LeftBracket => Key::LBracket,
            rdev::Key::RightBracket => Key::RBracket,
            rdev::Key::KeyA => Key::A,
            rdev::Key::KeyS => Key::S,
            rdev::Key::KeyD => Key::D,
            rdev::Key::KeyF => Key::F,
            rdev::Key::KeyG => Key::G,
            rdev::Key::KeyH => Key::H,
            rdev::Key::KeyJ => Key::J,
            rdev::Key::KeyK => Key::K,
            rdev::Key::KeyL => Key::L,
            rdev::Key::SemiColon => Key::SemiColon,
            rdev::Key::Quote => Key::Quote,
            rdev::Key::BackSlash => Key::Backslash,
            rdev::Key::IntlBackslash => todo!(),
            rdev::Key::KeyZ => Key::Z,
            rdev::Key::KeyX => Key::X,
            rdev::Key::KeyC => Key::C,
            rdev::Key::KeyV => Key::V,
            rdev::Key::KeyB => Key::B,
            rdev::Key::KeyN => Key::N,
            rdev::Key::KeyM => Key::M,
            rdev::Key::Comma => Key::Comma,
            rdev::Key::Dot => Key::Dot,
            rdev::Key::Slash => Key::Slash,
            rdev::Key::Insert => Key::Insert,
            rdev::Key::KpReturn => todo!(),
            rdev::Key::KpMinus => todo!(),
            rdev::Key::KpPlus => todo!(),
            rdev::Key::KpMultiply => todo!(),
            rdev::Key::KpDivide => todo!(),
            rdev::Key::Kp0 => todo!(),
            rdev::Key::Kp1 => todo!(),
            rdev::Key::Kp2 => todo!(),
            rdev::Key::Kp3 => todo!(),
            rdev::Key::Kp4 => todo!(),
            rdev::Key::Kp5 => todo!(),
            rdev::Key::Kp6 => todo!(),
            rdev::Key::Kp7 => todo!(),
            rdev::Key::Kp8 => todo!(),
            rdev::Key::Kp9 => todo!(),
            rdev::Key::KpDelete => todo!(),
            rdev::Key::Unknown(_) => Key::Unknown,
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_to_and_from_u8() {
        assert_eq!(Key::from(0u8), Key::Unknown);
        assert_eq!(Key::from(10u8), Key::J);
        assert_eq!(Key::from(200u8), Key::Unknown);
    }
}
