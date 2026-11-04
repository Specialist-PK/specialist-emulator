use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Cr,
    Bsp,
    Period,
    Colon,
    Minus,
    Clear,
    Lf,
    Slash,
    Backslash,
    H,
    Digit0,
    F11,
    Right,
    Comma,
    V,
    Z,
    Digit9,
    F10,
    Rpt,
    At,
    D,
    RBracket,
    Digit8,
    F9,
    Left,
    B,
    L,
    LBracket,
    Digit7,
    F8,
    Space,
    X,
    O,
    G,
    Digit6,
    F7,
    Esc,
    T,
    R,
    N,
    Digit5,
    F6,
    Tab,
    I,
    P,
    E,
    Digit4,
    F5,
    Down,
    M,
    A,
    K,
    Digit3,
    F4,
    Up,
    S,
    W,
    U,
    Digit2,
    F3,
    Home,
    Caret,
    Y,
    C,
    Digit1,
    F2,
    Lang,
    Q,
    F,
    J,
    Semicolon,
    F1,
    Shift,
}

impl FromStr for Key {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "cr" => Ok(Self::Cr),
            "bsp" => Ok(Self::Bsp),
            "period" => Ok(Self::Period),
            "colon" => Ok(Self::Colon),
            "minus" => Ok(Self::Minus),
            "clear" => Ok(Self::Clear),
            "lf" => Ok(Self::Lf),
            "slash" => Ok(Self::Slash),
            "backslash" => Ok(Self::Backslash),
            "h" => Ok(Self::H),
            "0" => Ok(Self::Digit0),
            "f11" => Ok(Self::F11),
            "right" => Ok(Self::Right),
            "comma" => Ok(Self::Comma),
            "v" => Ok(Self::V),
            "z" => Ok(Self::Z),
            "9" => Ok(Self::Digit9),
            "f10" => Ok(Self::F10),
            "rpt" => Ok(Self::Rpt),
            "at" => Ok(Self::At),
            "d" => Ok(Self::D),
            "rbracket" => Ok(Self::RBracket),
            "8" => Ok(Self::Digit8),
            "f9" => Ok(Self::F9),
            "left" => Ok(Self::Left),
            "b" => Ok(Self::B),
            "l" => Ok(Self::L),
            "lbracket" => Ok(Self::LBracket),
            "7" => Ok(Self::Digit7),
            "f8" => Ok(Self::F8),
            "space" => Ok(Self::Space),
            "x" => Ok(Self::X),
            "o" => Ok(Self::O),
            "g" => Ok(Self::G),
            "6" => Ok(Self::Digit6),
            "f7" => Ok(Self::F7),
            "esc" => Ok(Self::Esc),
            "t" => Ok(Self::T),
            "r" => Ok(Self::R),
            "n" => Ok(Self::N),
            "5" => Ok(Self::Digit5),
            "f6" => Ok(Self::F6),
            "tab" => Ok(Self::Tab),
            "i" => Ok(Self::I),
            "p" => Ok(Self::P),
            "e" => Ok(Self::E),
            "4" => Ok(Self::Digit4),
            "f5" => Ok(Self::F5),
            "down" => Ok(Self::Down),
            "m" => Ok(Self::M),
            "a" => Ok(Self::A),
            "k" => Ok(Self::K),
            "3" => Ok(Self::Digit3),
            "f4" => Ok(Self::F4),
            "up" => Ok(Self::Up),
            "s" => Ok(Self::S),
            "w" => Ok(Self::W),
            "u" => Ok(Self::U),
            "2" => Ok(Self::Digit2),
            "f3" => Ok(Self::F3),
            "home" => Ok(Self::Home),
            "caret" => Ok(Self::Caret),
            "y" => Ok(Self::Y),
            "c" => Ok(Self::C),
            "1" => Ok(Self::Digit1),
            "f2" => Ok(Self::F2),
            "lang" => Ok(Self::Lang),
            "q" => Ok(Self::Q),
            "f" => Ok(Self::F),
            "j" => Ok(Self::J),
            "semicolon" => Ok(Self::Semicolon),
            "f1" => Ok(Self::F1),
            "shift" => Ok(Self::Shift),
            _ => Err(value.to_owned()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: Key,
    pub pressed: bool,
    pub cycle: Option<u32>,
}

const KEY_MATRIX: [[Key; 6]; 12] = [
    [
        Key::Cr,
        Key::Bsp,
        Key::Period,
        Key::Colon,
        Key::Minus,
        Key::Clear,
    ],
    [
        Key::Lf,
        Key::Slash,
        Key::Backslash,
        Key::H,
        Key::Digit0,
        Key::F11,
    ],
    [
        Key::Right,
        Key::Comma,
        Key::V,
        Key::Z,
        Key::Digit9,
        Key::F10,
    ],
    [
        Key::Rpt,
        Key::At,
        Key::D,
        Key::RBracket,
        Key::Digit8,
        Key::F9,
    ],
    [
        Key::Left,
        Key::B,
        Key::L,
        Key::LBracket,
        Key::Digit7,
        Key::F8,
    ],
    [Key::Space, Key::X, Key::O, Key::G, Key::Digit6, Key::F7],
    [Key::Esc, Key::T, Key::R, Key::N, Key::Digit5, Key::F6],
    [Key::Tab, Key::I, Key::P, Key::E, Key::Digit4, Key::F5],
    [Key::Down, Key::M, Key::A, Key::K, Key::Digit3, Key::F4],
    [Key::Up, Key::S, Key::W, Key::U, Key::Digit2, Key::F3],
    [Key::Home, Key::Caret, Key::Y, Key::C, Key::Digit1, Key::F2],
    [Key::Lang, Key::Q, Key::F, Key::J, Key::Semicolon, Key::F1],
];

#[derive(Default)]
pub(crate) struct Keyboard {
    rows: [u8; 12],
    shift: bool,
}

impl Keyboard {
    pub(crate) fn set_key(&mut self, key: Key, pressed: bool) {
        if key == Key::Shift {
            self.shift = pressed;
            return;
        }
        if let Some((row, column)) = KEY_MATRIX.iter().enumerate().find_map(|(row, keys)| {
            keys.iter()
                .position(|candidate| *candidate == key)
                .map(|column| (row, column))
        }) {
            if pressed {
                self.rows[row] |= 1 << column;
            } else {
                self.rows[row] &= !(1 << column);
            }
        }
    }

    pub(crate) fn shift_pressed(&self) -> bool {
        self.shift
    }

    pub(crate) fn rows(&self, column_mask: u8) -> u16 {
        let mut pressed = 0;
        for mask in (0..6).map(|column| 1 << column) {
            if column_mask & mask == 0 {
                for (row, keys) in self.rows.iter().enumerate() {
                    pressed |= u16::from(keys & mask != 0) << row;
                }
            }
        }
        !pressed & 0x0fff
    }

    pub(crate) fn columns(&self, row_mask: u16) -> u8 {
        let mut pressed = 0;
        for (row, keys) in self.rows.iter().enumerate() {
            if row_mask & (1 << row) == 0 {
                pressed |= keys;
            }
        }
        !pressed & 0x3f
    }
}
