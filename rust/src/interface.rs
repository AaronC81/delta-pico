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
}

#[repr(C)]
pub struct DisplayInterface {
    pub width: u64,
    pub height: u64,

    pub fill_screen: extern "C" fn(c: u16),
    pub draw_char: extern "C" fn(x: i64, y: i64, character: u8),
    pub draw_line: extern "C" fn(x1: i64, y1: i64, x2: i64, y2: i64, c: u16),
    pub print: extern "C" fn(s: *const u8),
    pub set_cursor: extern "C" fn(x: i64, y: i64),
    pub draw: extern "C" fn(),
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
            Some(input)
        } else {
            None
        }
    }
}

/// All possible user inputs.
#[repr(C)]
pub enum ButtonInput {
    None,

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

    Add,
    Subtract,
    Multiply,
    Fraction,
}
