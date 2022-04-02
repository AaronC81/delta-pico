use core::fmt::Debug;

use alloc::{string::{String, ToString}, vec, vec::Vec};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Colour(pub u16);

impl Colour {
    pub const WHITE: Self = Self(0xFFFF);
    pub const BLACK: Self = Self(0x0000);
    pub const ORANGE: Self = Self(0xD340);
    pub const BLUE: Self = Self(0x0392);
    pub const DARK_BLUE: Self = Self(0x024B);
    pub const GREY: Self = Self(0x31A6);
    pub const RED: Self = Self(0xF800);
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ShapeFill {
    Filled,
    Hollow,
}

#[repr(C)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum FontSize {
    Default,
    Small,
}

pub trait DisplayInterface {
    type Sprite;

    fn width(&self) -> u16;
    fn height(&self) -> u16;

    fn new_sprite(&mut self, width: u16, height: u16) -> Self::Sprite;
    fn switch_to_sprite(&mut self, sprite: &mut Self::Sprite);
    fn switch_to_screen(&mut self);

    fn fill_screen(&mut self, c: Colour);
    fn draw_char(&mut self, character: u8);
    fn draw_line(&mut self, x1: i16, y1: i16, x2: i16, y2: i16, c: Colour);
    fn draw_rect(&mut self, x1: i16, y1: i16, w: u16, h: u16, c: Colour, fill: ShapeFill, radius: u16);
    fn draw_sprite(&mut self, x: i16, y: i16, sprite: &Self::Sprite);
    fn draw_bitmap(&mut self, x: i16, y: i16, name: &str);

    fn print(&mut self, s: &str);

    fn set_cursor(&mut self, x: i16, y: i16);
    fn get_cursor(&self) -> (i16, i16);
    fn set_font_size(&mut self, size: FontSize);
    fn get_font_size(&self) -> FontSize;

    fn draw(&mut self);

    // Helper methods
    fn print_at(&mut self, x: i16, y: i16, s: &str) {
        self.set_cursor(x, y);
        self.print(s);
    }

    fn print_centred(&mut self, x: i16, y: i16, w: u16, s: &str) {
        let (text_width, _) = self.string_size(s);

        let x_offset = (w as i16 - text_width) / 2;
        self.print_at(x + x_offset, y, s);
    }

    /// Calculates the size of a SINGLE-LINE string, returning it in the form (width, height).
    /// 
    /// The implementation of this function is *very* dodgy, deliberately drawing out-of-bounds and
    /// watching the cursor. It may not be portable depending on the HAL - if you start getting
    /// panics when printing, start looking here!
    fn string_size(&mut self, string: &str) -> (i16, i16) {
        // Won't work for strings with newlines

        // Draw the string off the screen and see how much the cursor moved
        // HACK: This draws to a buffer, right? Could we be overwriting some random memory by
        // writing out of bounds??
        self.set_cursor(0, self.height() as i16 + 100);
        self.print(string);
        let (x, _) = self.get_cursor();
        self.print("\n");
        let (_, y) = self.get_cursor();
        (x, y - (self.height() as i16 + 100))
    }

    /// Wraps a string by breaking it into lines on whitespace, so that it fits in a given width.
    /// Returns a tuple in the form:
    /// 
    /// (
    ///     list of lines,
    ///     height of each line,
    ///     total height of text,
    /// )
    fn wrap_text(&mut self, string: &str, width: u16) -> (Vec<String>, i16, i16) {
        // All characters are assumed to have the same height

        let mut x = 0;
        let mut y = 0;
        let mut lines: Vec<String> = vec!["".into()];
        let char_height = self.string_size("A").1;

        for word in string.split_whitespace() {
            // Work out size of this word
            let (this_char_x, this_char_y) = self.string_size(word);

            // Rare special case - what if this word will never fit on a single line?
            // I've only seen this in panic messages so far
            // VERY slow, but panic messages occur infrequently enough that I don't think that's a
            // huge problem
            if this_char_x > width as i16 {
                // For clarity (and ease of implementation!) start a new line for extremely long
                // words
                lines.push("".into());

                // Break it up character-by-character until we've exhausted the whole word
                let word_chars = word.chars();
                let mut buffer = String::new();

                for char in word_chars {
                    // Add character to buffer
                    buffer.push(char);
                    
                    // If the word no longer fits on a line, insert buffer minus last character as
                    // a line, start new line, and reset buffer
                    let (buffer_width, _) = self.string_size(&buffer);
                    if buffer_width > width as i16 {
                        buffer.remove(buffer.len() - 1);
                        lines.last_mut().unwrap().push_str(&buffer);

                        lines.push("".into());
                        y += this_char_y;

                        buffer = char.to_string();
                    } 
                }

                // We might be left with a buffer, empty it and set current X position
                lines.last_mut().unwrap().push_str(&buffer);
                lines.last_mut().unwrap().push(' ');

                x = self.string_size(&buffer).0;

                continue;
            }

            // Is it going to fit on this line?
            x += this_char_x;
            if x > width as i16 {
                // No - start a new line
                lines.push("".into());
                x = this_char_x;
                y += this_char_y;
            }
            
            // Add to end of current line
            lines.last_mut().unwrap().push_str(word);
            lines.last_mut().unwrap().push(' ');
        }

        // Factor in width of last line into height
        (lines, char_height, y + char_height)
    }

    /// Performs a set of draw operations with a different font size, and returns to the original
    /// font size at the end.
    fn with_font_size<T, F>(&mut self, size: FontSize, func: F) -> T where F : FnOnce() -> T {
        let original_size = self.get_font_size();
        self.set_font_size(size);
        let result = func();
        self.set_font_size(original_size);
        result
    }
}
