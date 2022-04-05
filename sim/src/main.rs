use std::{process::exit, time::SystemTime};

use delta_pico_rust::{interface::{ApplicationFramework, DisplayInterface, Colour, ButtonsInterface, ButtonEvent, StorageInterface, ButtonInput}, graphics::Sprite, delta_pico_main};
use minifb::{Window, WindowOptions, Scale, InputCallback, Key, KeyRepeat};

const STORAGE_SIZE: usize = 65536;

struct FrameworkImpl {
    window: Window,
    start_time: SystemTime,
    storage: [u8; STORAGE_SIZE],
}

impl ApplicationFramework for FrameworkImpl {
    type DisplayI = Self;
    type ButtonsI = Self;
    type StorageI = Self;

    fn display(&self) -> &Self::DisplayI { self }
    fn display_mut(&mut self) -> &mut Self::DisplayI { self }
    fn buttons(&self) -> &Self::ButtonsI { self }
    fn buttons_mut(&mut self) -> &mut Self::ButtonsI { self }
    fn storage(&self) -> &Self::StorageI { self }
    fn storage_mut(&mut self) -> &mut Self::StorageI { self }

    fn hardware_revision(&self) -> String {
        "Simulator".to_string()
    }

    fn reboot_into_bootloader(&mut self) -> ! {
        panic!("no bootloader on simulator")
    }

    fn millis(&self) -> u64 { self.micros() / 1000 }
    fn micros(&self) -> u64 { SystemTime::now().duration_since(self.start_time).unwrap().as_micros() as u64 }

    fn memory_usage(&self) -> (usize, usize) {
        (0, 0)
    }
}

impl DisplayInterface for FrameworkImpl {
    fn width(&self) -> u16 { 240 }
    fn height(&self) -> u16 { 320 }

    fn draw_display_sprite(&mut self, sprite: &Sprite) {
        let buffer = sprite.data.iter().map(|c| rgb565_to_rgba8888(*c)).collect::<Vec<_>>();
        self.window.update_with_buffer(&buffer, 240, 320);

        if !self.window.is_open() {
            exit(0);
        }
    }
}

fn rgb565_to_rgba8888(colour: Colour) -> u32 {
    let r5 = ((colour.0 as u32) & 0b1111100000000000) >> 11;
    let g6 = ((colour.0 as u32) & 0b0000011111100000) >> 5;
    let b5 = (colour.0 as u32) & 0b0000000000011111;

    let r8 = (r5 * 527 + 23) >> 6;
    let g8 = (g6 * 259 + 33) >> 6;
    let b8 = (b5 * 527 + 23) >> 6;

    (r8 << 16) | (g8 << 8) | b8
}

const BUTTON_MAPPING: [(bool, Key, ButtonInput); 25] = [
    (false, Key::Left, ButtonInput::MoveLeft),
    (false, Key::Right, ButtonInput::MoveRight),
    (false, Key::Up, ButtonInput::MoveUp),
    (false, Key::Down, ButtonInput::MoveDown),
    (false, Key::Enter, ButtonInput::Exe),
    (false, Key::Escape, ButtonInput::Menu),
    (false, Key::Space, ButtonInput::List),

    (false, Key::Key0, ButtonInput::Digit(0)),
    (false, Key::Key1, ButtonInput::Digit(1)),
    (false, Key::Key2, ButtonInput::Digit(2)),
    (false, Key::Key3, ButtonInput::Digit(3)),
    (false, Key::Key4, ButtonInput::Digit(4)),
    (false, Key::Key5, ButtonInput::Digit(5)),
    (false, Key::Key6, ButtonInput::Digit(6)),
    (false, Key::Key7, ButtonInput::Digit(7)),
    (false, Key::Key8, ButtonInput::Digit(8)),
    (false, Key::Key9, ButtonInput::Digit(9)),

    (false, Key::Tab, ButtonInput::Shift),

    (false, Key::LeftBracket, ButtonInput::Parentheses),
    (false, Key::RightBracket, ButtonInput::Parentheses),
    (false, Key::Slash, ButtonInput::Fraction),
    (true,  Key::Equal, ButtonInput::Add),
    (false, Key::Minus, ButtonInput::Subtract),
    (true,  Key::Key8, ButtonInput::Multiply),
    (false, Key::Backspace, ButtonInput::Delete),
];

impl ButtonsInterface for FrameworkImpl {
    fn wait_event(&mut self) -> ButtonEvent {
        loop {
            for (shifted, key, input) in BUTTON_MAPPING {
                let pressed_without_modifier = self.window.is_key_pressed(key, KeyRepeat::No);
                let shift_down = self.window.is_key_down(Key::LeftShift) || self.window.is_key_down(Key::RightShift);
                if pressed_without_modifier && ((!shifted && !shift_down) || (shifted && shift_down)) {
                    return ButtonEvent::Press(input)
                }
            }
            
            self.window.update();

            if !self.window.is_open() {
                exit(0);
            }
        }
    }

    fn poll_event(&mut self) -> Option<ButtonEvent> {
        None
    }
}

impl StorageInterface for FrameworkImpl {
    fn is_connected(&mut self) -> bool { true }
    fn is_busy(&mut self) -> bool { false }

    fn write(&mut self, address: u16, bytes: &[u8]) -> Option<()> {
        self.storage[(address as usize)..(address as usize + bytes.len())].copy_from_slice(bytes);
        Some(())
    }

    fn read(&mut self, address: u16, bytes: &mut [u8]) -> Option<()> {
        bytes.copy_from_slice(&self.storage[(address as usize)..(address as usize + bytes.len())]);
        Some(())
    }

    fn acquire_priority(&mut self) {}
    fn release_priority(&mut self) {}
}

fn main() {
    let mut framework = FrameworkImpl {
        window: Window::new(
            "Delta Pico",
            240,
            320,
            WindowOptions {
                resize: true,
                scale: Scale::X2,
                ..WindowOptions::default()
            },
        ).unwrap(),
        start_time: SystemTime::now(),
        storage: [0; STORAGE_SIZE],
    };

    delta_pico_main(framework);

    panic!("Main loop exited");
}
