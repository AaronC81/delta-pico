use crate::operating_system::{OSInput, os};

#[repr(C)]
#[derive(PartialEq, Eq)]
pub enum ButtonEvent {
    Press,
    Release,
}

#[repr(C)]
pub struct ButtonsInterface {
    pub wait_input_event: extern "C" fn(input: *mut ButtonInput, event: *mut ButtonEvent) -> bool,
    pub immediate_input_event: extern "C" fn(input: *mut ButtonInput, event: *mut ButtonEvent) -> bool,
}

impl ButtonsInterface {
    pub fn wait_press(&self) -> Option<OSInput> {
        self.press_func_wrapper(self.wait_input_event)
    }

    pub fn immediate_press(&self) -> Option<OSInput> {
        self.press_func_wrapper(self.immediate_input_event)
    }

    fn press_func_wrapper(&self, func: extern "C" fn(input: *mut ButtonInput, event: *mut ButtonEvent) -> bool) -> Option<OSInput> {
        // Garbage default values
        let mut input: ButtonInput = ButtonInput::None;
        let mut event: ButtonEvent = ButtonEvent::Release;

        if (func)(&mut input as *mut _, &mut event as *mut _) && event == ButtonEvent::Press {
            let mut result = Some(match input {
                // Special cases
                ButtonInput::Menu => {
                    os().toggle_menu();
                    return None
                }
                ButtonInput::Text => {
                    os().text_mode = !os().text_mode;
                    return None
                }
                ButtonInput::None => return None,

                // Straight key mappings
                ButtonInput::Exe => OSInput::Exe,
                ButtonInput::Shift => OSInput::Shift,
                ButtonInput::List => OSInput::List,

                ButtonInput::MoveLeft => OSInput::MoveLeft,
                ButtonInput::MoveRight => OSInput::MoveRight,
                ButtonInput::MoveUp => OSInput::MoveUp,
                ButtonInput::MoveDown => OSInput::MoveDown,
                ButtonInput::Delete => OSInput::Delete,

                ButtonInput::Digit0 => OSInput::Digit(0),
                ButtonInput::Digit1 => OSInput::Digit(1),
                ButtonInput::Digit2 => OSInput::Digit(2),
                ButtonInput::Digit3 => OSInput::Digit(3),
                ButtonInput::Digit4 => OSInput::Digit(4),
                ButtonInput::Digit5 => OSInput::Digit(5),
                ButtonInput::Digit6 => OSInput::Digit(6),
                ButtonInput::Digit7 => OSInput::Digit(7),
                ButtonInput::Digit8 => OSInput::Digit(8),
                ButtonInput::Digit9 => OSInput::Digit(9),

                ButtonInput::Point => OSInput::Point,
                ButtonInput::Parentheses => OSInput::Parentheses,
                ButtonInput::Add => OSInput::Add,
                ButtonInput::Subtract => OSInput::Subtract,
                ButtonInput::Multiply => OSInput::Multiply,
                ButtonInput::Fraction => OSInput::Fraction,
                ButtonInput::Power => OSInput::Power,
            });

            // Intercept if a digit was pressed in text mode - this needs to be converted to a
            // character according to the OS' multi-tap state
            if os().text_mode {
                if let Some(OSInput::Digit(d)) = result {
                    result = os().multi_tap.input(OSInput::Digit(d));
                } else {
                    // Make sure we don't cycle the wrong character if we e.g. move with the arrows
                    os().multi_tap.drop_keypress();
                }
            }

            result
        } else {
            None
        }
    }
}

/// All possible user inputs.
#[repr(C)]
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

    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,

    Point,
    Parentheses,

    Add,
    Subtract,
    Multiply,
    Fraction,
    Power,
}

impl ButtonInput {
    pub fn as_digit(&self) -> Option<u8> {
        match self {
            &Self::Digit0 => Some(0),
            &Self::Digit1 => Some(1),
            &Self::Digit2 => Some(2),
            &Self::Digit3 => Some(3),
            &Self::Digit4 => Some(4),
            &Self::Digit5 => Some(5),
            &Self::Digit6 => Some(6),
            &Self::Digit7 => Some(7),
            &Self::Digit8 => Some(8),
            &Self::Digit9 => Some(9),
            _ => None,
        }
    }
}