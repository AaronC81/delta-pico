use core::{convert::Infallible, fmt::Debug};

use alloc::{vec, vec::Vec, string::{String, ToString}};
use az::SaturatingAs;
use crate::{interface::{Colour, FontSize, ShapeFill}};

#[derive(Debug, Clone)]
pub struct Sprite {
    pub width: u16,
    pub height: u16,
    pub data: Vec<Colour>,

    pub cursor_x: i16,
    pub cursor_y: i16,
    pub font: &'static dyn AsciiFont,
}

impl Sprite {
    pub fn new(width: u16, height: u16) -> Sprite {
        Sprite {
            width,
            height,
            data: vec![Colour::BLACK; width as usize * height as usize],
            cursor_x: 0,
            cursor_y: 0,
            font: &crate::font_data::DroidSans20,
        }
    }

    pub fn empty() -> Sprite {
        Sprite::new(0, 0)
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.data.resize(width as usize * height as usize, Colour::BLACK);
        self.fill(Colour::BLACK);
    }

    pub fn pixel(&mut self, x: u16, y: u16) -> &mut Colour {
        &mut self.data[y as usize * self.width as usize + x as usize]
    }

    pub fn pixel_immutable(&self, x: u16, y: u16) -> Colour {
        self.data[y as usize * self.width as usize + x as usize]
    }

    pub fn try_pixel(&mut self, x: i16, y: i16) -> Option<&mut Colour> {
        let index = y as isize * self.width as isize + x as isize;

        if index < 0 || index as usize >= self.data.len() {
            None
        } else {
            Some(self.pixel(x as u16, y as u16))
        }
    }

    pub fn try_pixel_immutable(&self, x: i16, y: i16) -> Option<Colour> {
        let index = y as isize * self.width as isize + x as isize;

        if index < 0 || index as usize >= self.data.len() {
            None
        } else {
            Some(self.pixel_immutable(x as u16, y as u16))
        }
    }

    pub fn fill(&mut self, colour: Colour) {
        self.data.fill(colour);
    }
    
    pub fn draw_pixel(&mut self, x: i16, y: i16, colour: Colour) {
        if let Some(px) = self.try_pixel(x, y) {
            *px = colour;
        }
    }

    pub fn draw_line(&mut self, mut x1: i16, mut y1: i16, mut x2: i16, mut y2: i16, colour: Colour) {
        // We expect the 1s to be lower than the 2s - if not, swap them
        if x1 > x2 { core::mem::swap(&mut x1, &mut x2); }
        if y1 > y2 { core::mem::swap(&mut y1, &mut y2); }

        // Only horizontal and vertical lines supported, but the OS doesn't need to draw anything else
        if y1 == y2 {
            // Horizontal
            for x in x1..x2 {
                self.draw_pixel(x, y1, colour);
            }
        } else if x1 == x2 {
            // Vertical
            for y in y1..y2 {
                self.draw_pixel(x1, y, colour);
            }
        }
    }

    pub fn draw_rect(&mut self, x: i16, y: i16, mut w: u16, mut h: u16, colour: Colour, filled: ShapeFill, radius: u16) {
        // TODO: radius
        
        // If the rectangle spills over the left, adjust width and X origin so we still start
        // in-bounds (which also allows `x` to become unsigned)
        let x = if x < 0 {
            w -= x.abs() as u16;
            0usize
        } else {
            x as usize
        };

        // Same if it spills over the right
        if x as u16 + w >= self.width {
            w = self.width - x as u16;
        }

        // Same for spilling over the top
        let y = if y < 0 {
            h -= y.abs() as u16;
            0usize
        } else {
            y as usize
        };

        // Spilling over the bottom
        if y as u16 + h >= self.height {
            h = self.height - y as u16;
        }

        // Draw line-by-line
        for curr_y in y..(y + h as usize) {
            if curr_y < 0 { continue; }
            let curr_y = curr_y as usize;

            if filled == ShapeFill::Filled {
                self.data[
                    (curr_y * self.width as usize + x)
                    ..(curr_y * self.width as usize + x + w as usize)
                ].fill(colour);
            } else {
                // Only draw on a border
                for curr_x in x..(x + w as usize) {
                    if curr_x == x || curr_y == y || curr_x == (x + w as usize - 1) || curr_y == (y + h as usize - 1) {
                        self.draw_pixel(curr_x as i16, curr_y as i16, colour);
                    }
                }
            }
        }
    }

    pub fn draw_char_at(&mut self, x: i16, y: i16, character: char) {
        self.set_cursor(x, y);
        self.draw_char(character);
    }

    pub fn draw_char(&mut self, character: char) {
        let (x, y) = self.get_cursor();

        if character == '\n' {
            self.cursor_x = 0;
            self.cursor_y += self.font.char_data('A' as u8).unwrap()[1] as i16;
            return;
        }

        let character_bitmap = self.font.char_data(character as u8);
        if character_bitmap.is_none() { return; }
        let character_bitmap = character_bitmap.unwrap();

        // Each character is 4bpp;, so we maintain a flip-flopping boolean of whether to read the
        // upper or lower byte
        let mut lower_byte = false;
        let mut index = 2usize;

        let width = character_bitmap[0];
        let height = character_bitmap[1];

        for ox in 0..width {
            for oy in 0..height {
                let alpha_nibble = if lower_byte {
                    lower_byte = false;
                    let x = character_bitmap[index] & 0xF;
                    index += 1;
                    x
                } else {
                    lower_byte = true;
                    (character_bitmap[index] & 0xF0) >> 4
                };

                // Don't need to draw if it's totally transparent
                if alpha_nibble == 0 { continue; }

                if let Some(background) = self.try_pixel_immutable(x + ox as i16, y + oy as i16) {
                    self.draw_pixel(
                        x + ox as i16,
                        y + oy as i16, 
                        background.clone().interpolate_with_nibble(Colour::WHITE, alpha_nibble),
                    );
                }
            }
        }

        self.cursor_x += Into::<i16>::into(character_bitmap[0]) - 1;
    } 

    pub fn draw_sprite(&mut self, x: i16, y: i16, sprite: &Sprite) {
        for x_offset in 0..sprite.width {
            for y_offset in 0..sprite.height {
                self.draw_pixel(
                    x + x_offset.saturating_as::<i16>(),
                    y + y_offset.saturating_as::<i16>(),
                    sprite.pixel_immutable(x_offset, y_offset)
                );
            }
        }
    }

    pub fn draw_bitmap(&mut self, x: i16, y: i16, name: &str) {
        // Look up bitmap
        let bitmap = crate::bitmap_data::lookup(name);

        let width = bitmap[0];
        let height = bitmap[1];
        let transparency = bitmap[2];
        let run_length = bitmap[3];
    
        let mut index = 4;
        let mut ox = 0;
        while ox < width {
            let mut oy = 0;
            while oy < height {
                if bitmap[index] == run_length {
                    let times = bitmap[index + 1];
                    let colour = bitmap[index + 2];

                    if colour != transparency {
                        for i in 0..times {
                            self.draw_pixel(x + ox as i16, y + oy as i16 + i as i16, Colour(colour));
                        }
                    }

                    oy += times - 1;
                    index += 3;
                } else {
                    let colour = bitmap[index];
                    if colour != transparency {
                        self.draw_pixel(x + ox as i16, y + oy as i16, Colour(colour).into());
                    }
                    index += 1;
                }

                oy += 1;
            }

            ox += 1;
        }
    }
    
    pub fn print(&mut self, s: &str) {
        for c in s.chars() {
            self.draw_char(c);
        }
    }

    pub fn set_cursor(&mut self, x: i16, y: i16) {
        self.cursor_x = x;
        self.cursor_y = y;
    }

    pub fn get_cursor(&self) -> (i16, i16) {
        (self.cursor_x, self.cursor_y)
    }

    pub fn print_at(&mut self, x: i16, y: i16, s: &str) {
        self.set_cursor(x, y);
        self.print(s);
    }

    pub fn print_centred(&mut self, x: i16, y: i16, w: u16, s: &str) {
        let (text_width, _) = self.string_size(s);

        let x_offset = (w as i16 - text_width) / 2;
        self.print_at(x + x_offset, y, s);
    }

    /// Calculates the size of a SINGLE-LINE string, returning it in the form (width, height).
    /// 
    /// The implementation of this function is *very* dodgy, deliberately drawing out-of-bounds and
    /// watching the cursor. It may not be portable depending on the HAL - if you start getting
    /// panics when printing, start looking here!
    pub fn string_size(&mut self, string: &str) -> (i16, i16) {
        // Won't work for strings with newlines

        // Draw the string off the screen and see how much the cursor moved
        // HACK: This draws to a buffer, right? Could we be overwriting some random memory by
        // writing out of bounds??
        self.set_cursor(0, self.height as i16 + 100);
        self.print(string);
        let (x, _) = self.get_cursor();
        self.print("\n");
        let (_, y) = self.get_cursor();
        (x, y - (self.height as i16 + 100))
    }

    /// Wraps a string by breaking it into lines on whitespace, so that it fits in a given width.
    /// Returns a tuple in the form:
    /// 
    /// (
    ///     list of lines,
    ///     height of each line,
    ///     total height of text,
    /// )
    pub fn wrap_text(&mut self, string: &str, width: u16) -> (Vec<String>, i16, i16) {
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

    // TODO: actual FontSize implementation
    pub fn with_font<T, F>(&mut self, font: &'static dyn AsciiFont, func: F) -> T where F : FnOnce(&mut Self) -> T {
        let original_font = self.font;
        self.font = font;
        let result = func(self);
        self.font = original_font;
        result
    }
}

pub trait AsciiFont: Debug {
    fn char_data(&self, c: u8) -> Option<&'static [u8]>;
}
