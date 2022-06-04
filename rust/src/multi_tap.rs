use crate::{operating_system::{OSInput, OperatingSystem, os_accessor, OperatingSystemPointer}, interface::{ButtonInput, ApplicationFramework}};

pub struct MultiTapState<F: ApplicationFramework + 'static> {
    pub os: OperatingSystemPointer<F>,
    current_list: Option<&'static [char]>,
    current_index: Option<usize>,
    current_digit: Option<u8>,
    current_shifted: bool,
    last_press_ms: u64,
}

os_accessor!(MultiTapState<F>);

const PRESS_COOLDOWN_MS: u64 = 750;

// These aren't in the same order as a phone keypad, because our keys are the other way around
//     Phone         Delta Pico
//   1   2   3       7   8   9
//   4   5   6       4   5   6
//   7   8   9       1   2   3
//   *   0   #       0   .   ^
const EIGHT_CHAR_LIST: [char; 3] = ['a', 'b', 'c'];
const NINE_CHAR_LIST:  [char; 3] = ['d', 'e', 'f'];
const FOUR_CHAR_LIST:  [char; 3] = ['g', 'h', 'i'];
const FIVE_CHAR_LIST:  [char; 3] = ['j', 'k', 'l'];
const SIX_CHAR_LIST:   [char; 3] = ['m', 'n', 'o'];
const ONE_CHAR_LIST:   [char; 4] = ['p', 'q', 'r', 's'];
const TWO_CHAR_LIST:   [char; 3] = ['t', 'u', 'v'];
const THREE_CHAR_LIST: [char; 4] = ['w', 'x', 'y', 'z'];

impl<F: ApplicationFramework> MultiTapState<F> {
    pub fn new(os: OperatingSystemPointer<F>) -> Self {
        Self {
            os,
            current_index: None,
            current_list: None,
            current_digit: None,
            current_shifted: false,
            last_press_ms: 0,
        }
    }

    pub fn input(&mut self, input: OSInput) -> Option<OSInput> {
        let shift = matches!(input, OSInput::ShiftedButton(_));

        if let OSInput::Button(ButtonInput::Digit(digit)) | OSInput::ShiftedButton(ButtonInput::Digit(digit)) = input {
            // If it's been more than the threshold time since a key was pressed, discard the
            // information about the previous keypress and start a new character
            let now_ms = self.os().framework.millis();
            if now_ms - self.last_press_ms > PRESS_COOLDOWN_MS {
                self.drop_keypress();
            }
            self.last_press_ms = now_ms;

            // Did the user press the same digit again?
            if let Some(current_digit) = &mut self.current_digit {
                if digit == *current_digit {
                    // Increment current list index, wrapping if necessary
                    self.current_index = Some(
                        (self.current_index.unwrap() + 1) % self.current_list.unwrap().len()
                    );

                    // Replace last character in string with new one
                    let mut new_char = self.current_list.unwrap()[self.current_index.unwrap()];
                    if self.current_shifted {
                        new_char = new_char.to_ascii_uppercase();
                    }

                    return Some(OSInput::TextMultiTapCycle(new_char));
                } 
            } 

            // If we didn't return, we pressed our first digit, or a different digit than the
            // last - switch to new digit list
            self.current_list = Some(match digit {
                0 => return None,
                7 => return None,

                1 => &ONE_CHAR_LIST,
                2 => &TWO_CHAR_LIST,
                3 => &THREE_CHAR_LIST,
                4 => &FOUR_CHAR_LIST,
                5 => &FIVE_CHAR_LIST,
                6 => &SIX_CHAR_LIST,
                8 => &EIGHT_CHAR_LIST,
                9 => &NINE_CHAR_LIST,

                _ => unreachable!(),
            });
            self.current_index = Some(0);
            self.current_digit = Some(digit);
            self.current_shifted = shift;

            // Insert new character
            let mut new_char = self.current_list.unwrap()[0];
            if shift {
                new_char = new_char.to_ascii_uppercase();
            }

            return Some(OSInput::TextMultiTapNew(new_char));
        }

        None
    }

    pub fn drop_keypress(&mut self) {
        self.current_list = None;
        self.current_digit = None;
        self.current_index = None;
        self.current_shifted = false;
    }
}
