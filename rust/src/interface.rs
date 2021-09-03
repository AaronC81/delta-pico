use alloc::{format, string::{String, ToString}, vec, vec::{Vec}};
use rbop::render::CalculatedPoint;

use crate::{debug, operating_system::os};

static mut FRAMEWORK: *mut ApplicationFrameworkInterface = 0 as *mut _;
pub fn framework() -> &'static mut ApplicationFrameworkInterface {
    unsafe {
        FRAMEWORK.as_mut().unwrap()
    }
}

#[no_mangle]
pub extern "C" fn delta_pico_set_framework(fw: *mut ApplicationFrameworkInterface) {
    unsafe {
        FRAMEWORK = fw;
    }
}

#[repr(C)]
pub struct ApplicationFrameworkInterface {
    pub panic_handler: extern "C" fn(*const u8) -> (),
    pub debug_handler: extern "C" fn(*const u8) -> (),
    pub display: DisplayInterface,
    pub buttons: ButtonsInterface,
    pub storage: StorageInterface,

    // Bit of a hack to have these here... ah well
    pub rbop_location_x: u64,
    pub rbop_location_y: u64,
}

#[repr(C)]
pub struct DisplayInterface {
    pub width: u64,
    pub height: u64,

    pub fill_screen: extern "C" fn(c: u16),
    pub draw_char: extern "C" fn(x: i64, y: i64, character: u8),
    pub draw_line: extern "C" fn(x1: i64, y1: i64, x2: i64, y2: i64, c: u16),
    pub draw_rect: extern "C" fn(x1: i64, y1: i64, w: i64, h: i64, c: u16, fill: bool, radius: u16),

    pub print: extern "C" fn(s: *const u8),
    pub set_cursor: extern "C" fn(x: i64, y: i64),
    pub get_cursor: extern "C" fn(x: *mut i64, y: *mut i64),

    pub draw: extern "C" fn(),
}

impl DisplayInterface {
    pub fn print(&self, s: impl Into<String>) {
        let mut bytes = s.into().as_bytes().to_vec();
        bytes.push(0);
        (self.print)(bytes.as_ptr())
    }

    pub fn get_cursor(&self) -> (i64, i64) {
        let mut x: i64 = 0;
        let mut y: i64 = 0;

        (self.get_cursor)(&mut x as *mut _, &mut y as *mut _);
        (x, y)
    }

    pub fn string_size(&self, string: impl Into<String>) -> (i64, i64) {
        // Won't work for strings with newlines

        // Draw the string off the screen and see how much the cursor moved
        // HACK: This draws to a buffer, right? Could we be overwriting some random memory by
        // writing out of bounds??
        (self.set_cursor)(0, self.height as i64 + 100);
        self.print(string);
        let (x, _) = self.get_cursor();
        self.print("\n");
        let (_, y) = self.get_cursor();
        (x, y - (self.height as i64 + 100))
    }

    pub fn wrap_text(&self, string: impl Into<String>, width: i64) -> (Vec<String>, i64, i64) {
        // All characters are assumed to have the same height

        let mut x = 0;
        let mut y = 0;
        let mut lines: Vec<String> = vec!["".into()];
        let char_height = self.string_size("A").1;

        for word in Into::<String>::into(string).split_whitespace() {
            let (this_char_x, this_char_y) = self.string_size(word.to_string());
            x += this_char_x;
            if x > width {
                lines.push("".into());
                x = this_char_x;
                y += this_char_y;
            }

            lines.last_mut().unwrap().push_str(word);
            lines.last_mut().unwrap().push(' ');
        }

        // Factor in width of last line into height
        (lines, char_height, y + char_height)
    }
}

#[repr(C)]
#[derive(PartialEq, Eq)]
pub enum ButtonEvent {
    Press,
    Release,
}

#[repr(C)]
pub struct ButtonsInterface {
    pub poll_input_event: extern "C" fn(input: *mut ButtonInput, event: *mut ButtonEvent) -> bool,
}

impl ButtonsInterface {
    pub fn poll_press(&self) -> Option<ButtonInput> {
        // Garbage default values
        let mut input: ButtonInput = ButtonInput::None;
        let mut event: ButtonEvent = ButtonEvent::Release;

        if (self.poll_input_event)(&mut input as *mut _, &mut event as *mut _) && event == ButtonEvent::Press {
            if input == ButtonInput::Menu {
                os().toggle_menu();
                None
            } else {
                Some(input)
            }
        } else {
            None
        }
    }
}

/// All possible user inputs.
#[repr(C)]
#[derive(PartialEq, Eq)]
pub enum ButtonInput {
    None,

    Menu,
    Exe,
    Shift,
    List,

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
    LeftParen,
    RightParen,

    Add,
    Subtract,
    Multiply,
    Fraction,
}

#[repr(C)]
pub struct StorageInterface {
    pub connected: extern "C" fn() -> bool,
    pub busy: extern "C" fn() -> bool,
    pub write: extern "C" fn(address: u16, count: u8, buffer: *const u8) -> bool,
    pub read: extern "C" fn(address: u16, count: u8, buffer: *mut u8) -> bool,
}

impl StorageInterface {
    pub const BYTES: usize = 65536;

    pub fn read(&self, address: u16, count: u8) -> Option<Vec<u8>> {
        let mut buffer = vec![0; count as usize];
        if (self.read)(address, count, buffer.as_mut_ptr()) {
            Some(buffer)
        } else {
            None
        }
    }

    pub fn write(&self, address: u16, bytes: &[u8]) -> Option<()> {
        if (self.write)(address, bytes.len() as u8, bytes.as_ptr()) {
            Some(())
        } else {
            None
        }
    }

    pub fn clear(&self) -> Option<()> {
        const CHUNK_SIZE: usize = 64;
        let buffer = [0; CHUNK_SIZE];
        for i in 0..(Self::BYTES / CHUNK_SIZE) {
            self.write((i * CHUNK_SIZE) as u16, &buffer)?;
        }
        Some(())
    }
}
