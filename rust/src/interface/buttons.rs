#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum ButtonInput {
    None,

    Menu,
    Exe,
    Shift,
    List,
    Text,

    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Delete,
    Clear,

    Digit(u8),

    Point,
    Parentheses,

    Add,
    Subtract,
    Multiply,
    Fraction,
    Power,
    Sqrt,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ButtonEvent {
    Press(ButtonInput),
    Release(ButtonInput),
}

pub trait ButtonsInterface {
    fn wait_event(&mut self) -> ButtonEvent;
    fn poll_event(&mut self) -> Option<ButtonEvent>;
}
