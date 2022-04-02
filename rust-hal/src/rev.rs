use delta_pico_rust::interface::ButtonInput as I;

#[cfg(delta_pico_rev = "rev3")]
mod config {
    pub const REVISION_NAME: &str = "Rev. 3";
}

pub use config::*;

pub const BUTTON_MAPPING: [[I; 7]; 7] = [
    [I::MoveUp, I::MoveRight, I::Menu, I::List, I::None, I::None, I::None],
    [I::MoveLeft, I::MoveDown, I::Shift, I::Text, I::None, I::None, I::Parentheses],
    [I::Digit(7), I::Digit(8), I::Digit(9), I::Delete, I::None, I::None, I::Clear],
    [I::Digit(4), I::Digit(5), I::Digit(6), I::Multiply, I::None, I::None, I::Fraction],
    [I::None, I::None, I::None, I::None, I::None, I::None, I::None],
    [I::Digit(0), I::Point, I::Power, I::None, I::None, I::None, I::Exe],
    [I::Digit(1), I::Digit(2), I::Digit(3), I::Add, I::None, I::None, I::Subtract],  
];

