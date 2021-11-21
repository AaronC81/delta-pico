use core::fmt::Debug;

use alloc::{string::{String, ToString}, vec, vec::Vec};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Sprite(*mut u8);

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Colour(pub u16);

impl Colour {
    pub const WHITE: Self = Self(0xFFFF);
    pub const BLACK: Self = Self(0x0000);
    pub const ORANGE: Self = Self(0xD340);
    pub const BLUE: Self = Self(0x0392);
    pub const DARK_BLUE: Self = Self(0x024B);
    pub const GREY: Self = Self(0x31A6);
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ShapeFill {
    Filled,
    Hollow,
}

#[repr(C)]
pub struct DisplayInterface {
    pub width: u64,
    pub height: u64,

    new_sprite: extern "C" fn(width: i16, height: i16) -> *mut u8,
    free_sprite: extern "C" fn(*mut u8),
    switch_to_sprite: extern "C" fn(*mut u8),
    switch_to_screen: extern "C" fn(),

    fill_screen: extern "C" fn(c: u16),
    draw_char: extern "C" fn(x: i64, y: i64, character: u8),
    draw_line: extern "C" fn(x1: i64, y1: i64, x2: i64, y2: i64, c: u16),
    draw_rect: extern "C" fn(x1: i64, y1: i64, w: i64, h: i64, c: u16, fill: bool, radius: u16),
    draw_sprite: extern "C" fn(x: i64, y: i64, sprite: *mut u8),
    draw_bitmap: extern "C" fn(x: i64, y: i64, name: *const u8),

    print: extern "C" fn(s: *const u8),
    set_cursor: extern "C" fn(x: i64, y: i64),
    get_cursor: extern "C" fn(x: *mut i64, y: *mut i64),

    draw: extern "C" fn(),
}

impl DisplayInterface {
    /// Creates a new sprite with a given size and returns it. The sprite must be freed manually
    /// using `free_sprite`.
    pub fn new_sprite(&self, width: u16, height: u16) -> Sprite {
        Sprite((self.new_sprite)(width as i16, height as i16))
    }

    /// Frees an allocated sprite. After this, the sprite cannot be used.
    pub fn free_sprite(&self, sprite: Sprite) {
        (self.free_sprite)(sprite.0)
    }

    /// Makes future drawing calls apply to the given sprite, until the screen is targeted again
    /// using `switch_to_screen`.
    pub fn switch_to_sprite(&self, sprite: &Sprite) {
        (self.switch_to_sprite)(sprite.0)
    }

    /// Makes future drawing calls apply to the screen. This is the default, so this method only
    /// needs to be called if you have switched to a sprite using `switch_to_sprite`.
    pub fn switch_to_screen(&self) {
        (self.switch_to_screen)()
    }

    /// Fills the screen with a colour.
    pub fn fill_screen(&self, colour: Colour) {
        (self.fill_screen)(colour.0)
    }

    /// Draws a single ASCII character to the screen.
    pub fn draw_char(&self, x: i64, y: i64, char: char) {
        (self.draw_char)(x, y, char as u8);
    }

    /// Draws a line to the screen.
    pub fn draw_line(&self, x1: i64, y1: i64, x2: i64, y2: i64, colour: Colour) {
        (self.draw_line)(x1, y1, x2, y2, colour.0);
    }

    /// Draws a rectangle to the screen.
    pub fn draw_rect(&self, x: i64, y: i64, width: i64, height: i64, colour: Colour, fill: ShapeFill, radius: u16) {
        (self.draw_rect)(x, y, width, height, colour.0, fill == ShapeFill::Filled, radius)
    }

    /// Draws a bitmap to the screen, by the bitmap's name.
    pub fn draw_bitmap(&self, x: i64, y: i64, name: &str) {
        let mut bytes = name.as_bytes().to_vec();
        bytes.push(0);
        (self.draw_bitmap)(x, y, bytes.as_ptr());
    }

    /// Draws a sprite to the screen.
    pub fn draw_sprite(&self, x: i64, y: i64, sprite: &Sprite) {
        (self.draw_sprite)(x, y, sprite.0);
    }

    /// Prints a string to the screen at the current cursor position.
    pub fn print(&self, s: &str) {
        let mut bytes = s.as_bytes().to_vec();
        bytes.push(0);
        (self.print)(bytes.as_ptr())
    }

    /// Moves the cursor, then prints a string.
    pub fn print_at(&self, x: i64, y: i64, s: &str) {
        (self.set_cursor)(x, y);
        self.print(s);
    }

    /// Prints a string horizontally centred inside a box with a given position and width, by
    /// calculating the width of the string and moving the cursor accordingly.
    pub fn print_centred(&self, x: i64, y: i64, w: i64, s: &str) {
        let (text_width, _) = self.string_size(&s);

        let x_offset = (w - text_width) / 2;
        self.print_at(x + x_offset, y, s);
    }

    /// Gets the cursor position in the form (x, y).
    pub fn get_cursor(&self) -> (i64, i64) {
        let mut x: i64 = 0;
        let mut y: i64 = 0;

        (self.get_cursor)(&mut x as *mut _, &mut y as *mut _);
        (x, y)
    }

    /// Sets the cursor position.
    pub fn set_cursor(&self, x: i64, y: i64) {
        (self.set_cursor)(x, y)
    }

    /// Commits the current drawing to the screen, showing the user the updated screen.
    pub fn draw(&self) {
        (self.draw)();
    }

    /// Calculates the size of a SINGLE-LINE string, returning it in the form (width, height).
    /// 
    /// The implementation of this function is *very* dodgy, deliberately drawing out-of-bounds and
    /// watching the cursor. It may not be portable depending on the HAL - if you start getting
    /// panics when printing, start looking here!
    pub fn string_size(&self, string: &str) -> (i64, i64) {
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

    /// Wraps a string by breaking it into lines on whitespace, so that it fits in a given width.
    /// Returns a tuple in the form:
    /// 
    /// (
    ///     list of lines,
    ///     total width,
    ///     total height,
    /// )
    pub fn wrap_text(&self, string: &str, width: i64) -> (Vec<String>, i64, i64) {
        // All characters are assumed to have the same height

        let mut x = 0;
        let mut y = 0;
        let mut lines: Vec<String> = vec!["".into()];
        let char_height = self.string_size("A").1;

        for word in Into::<String>::into(string).split_whitespace() {
            let (this_char_x, this_char_y) = self.string_size(&word.to_string());
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

