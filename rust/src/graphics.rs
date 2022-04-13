use core::{fmt::Debug, mem::swap};

use alloc::{vec, vec::Vec, string::{String, ToString}};
use az::SaturatingAs;
use num_traits::float::FloatCore;
use crate::{interface::{Colour, ShapeFill}};

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
        // We can optimise if the line is horizontal or vertical
        if y1 == y2 {
            // Horizontal
            if x1 > x2 { swap(&mut x1, &mut x2); }
            for x in x1..x2 {
                self.draw_pixel(x, y1, colour);
            }
        } else if x1 == x2 {
            // Vertical
            if y1 > y2 { swap(&mut y1, &mut y2); }
            for y in y1..y2 {
                self.draw_pixel(x1, y, colour);
            }
        } else {
            // It's a slanty line!
            // Employing Wu's algorithm, translated from Wikipedia
            // https://en.wikipedia.org/wiki/Xiaolin_Wu%27s_line_algorithm
            let steep = (y2 - y1).abs() > (x2 - x1).abs();

            if steep {
                swap(&mut x1, &mut y1);
                swap(&mut x2, &mut y2);
            }
            if x1 > x2 {
                swap(&mut x1, &mut x2);
                swap(&mut y1, &mut y2);
            }

            let dx = x2 - x1;
            let dy = y2 - y1;

            let gradient = if dx == 0 {
                1.0
            } else {
                dy as f32 / dx as f32
            };

            // First endpoint
            let x_end = x1 as f32;
            let y_end = y1 as f32 + gradient * (x_end - x1 as f32);
            let x_gap = (1 - x1) as f32;
            let x_pixel_1 = x_end;
            let y_pixel_1 = y_end.trunc();
            if steep {
                self.draw_interpolated_pixel(y_pixel_1.floor() as i16, x_pixel_1.floor() as i16, colour, (1.0 - y_end.fract()) * x_gap);
                self.draw_interpolated_pixel(y_pixel_1.floor() as i16 + 1, x_pixel_1.floor() as i16, colour, y_end.fract() * x_gap);
            } else {
                self.draw_interpolated_pixel(x_pixel_1.floor() as i16, y_pixel_1.floor() as i16, colour, (1.0 - y_end.fract()) * x_gap);
                self.draw_interpolated_pixel(x_pixel_1.floor() as i16 + 1, y_pixel_1.floor() as i16, colour, y_end.fract() * x_gap);
            }
            let mut inter_y = y_end + gradient;

            // Second endpoint
            let x_end = x2 as f32;
            let y_end = y2 as f32 + gradient * (x_end - x2 as f32);
            let x_gap = (x1 as f32 + 0.5).fract();
            let x_pixel_2 = x_end;
            let y_pixel_2 = y_end.trunc();
            if steep {
                self.draw_interpolated_pixel(y_pixel_2.floor() as i16, x_pixel_2.floor() as i16, colour, (1.0 - y_end.fract()) * x_gap);
                self.draw_interpolated_pixel(y_pixel_2.floor() as i16 + 1, x_pixel_2.floor() as i16, colour, y_end.fract() * x_gap);
            } else {
                self.draw_interpolated_pixel(x_pixel_2.floor() as i16, y_pixel_2.floor() as i16, colour, (1.0 - y_end.fract()) * x_gap);
                self.draw_interpolated_pixel(x_pixel_2.floor() as i16 + 1, y_pixel_2.floor() as i16, colour, y_end.fract() * x_gap);
            }

            // Main loop
            for x in (x_pixel_1 + 1.0).trunc() as i16 ..= (x_pixel_2 - 1.0).trunc() as i16 {
                if steep {
                    self.draw_interpolated_pixel(inter_y.trunc() as i16, x, colour, 1.0 - inter_y.fract());
                    self.draw_interpolated_pixel(inter_y.trunc() as i16 + 1, x, colour, inter_y.fract());
                } else {
                    self.draw_interpolated_pixel(x, inter_y.trunc() as i16, colour, 1.0 - inter_y.fract());
                    self.draw_interpolated_pixel(x, inter_y.trunc() as i16 + 1, colour, inter_y.fract());
                }
                inter_y += gradient;
            }
        }
    }

    pub fn draw_interpolated_pixel(&mut self, x: i16, y: i16, colour: Colour, ratio: f32) {
        if let Some(background) = self.try_pixel_immutable(x, y) {
            let pixel = background.interpolate_with_nibble(colour, (ratio * 15.0).ceil() as u8);
            self.draw_pixel(x, y, pixel);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_rect(&mut self, x: i16, y: i16, mut w: u16, mut h: u16, colour: Colour, filled: ShapeFill, _radius: u16) {
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
            self.cursor_y += self.font.char_data(b'A').unwrap().height as i16;
            return;
        }

        let character_bitmap = self.font.char_data(character as u8);
        if character_bitmap.is_none() { return; }
        let character_bitmap = character_bitmap.unwrap();

        // Each character is 4bpp;, so we maintain a flip-flopping boolean of whether to read the
        // upper or lower byte
        let mut lower_byte = false;
        let mut index = 0usize;

        let width = character_bitmap.width;
        let height = character_bitmap.height;

        for ox in 0..width {
            for oy in 0..height {
                let alpha_nibble = if lower_byte {
                    lower_byte = false;
                    let x = character_bitmap.data[index] & 0xF;
                    index += 1;
                    x
                } else {
                    lower_byte = true;
                    (character_bitmap.data[index] & 0xF0) >> 4
                };

                // Don't need to draw if it's totally transparent
                if alpha_nibble == 0 { continue; }

                if let Some(background) = self.try_pixel_immutable(x + ox as i16, y + oy as i16) {
                    self.draw_pixel(
                        x + ox as i16,
                        y + oy as i16, 
                        background.interpolate_with_nibble(Colour::WHITE, alpha_nibble),
                    );
                }
            }
        }

        self.cursor_x += width as i16 - 1;
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
                            self.draw_pixel(x + ox as i16, y + oy as i16 + i as i16, Colour::from_rgb565(colour));
                        }
                    }

                    oy += times - 1;
                    index += 3;
                } else {
                    let colour = bitmap[index];
                    if colour != transparency {
                        self.draw_pixel(x + ox as i16, y + oy as i16, Colour::from_rgb565(colour));
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
        let (text_width, _) = self.font.string_size(s);

        let x_offset = (w as i16 - text_width) / 2;
        self.print_at(x + x_offset, y, s);
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
        let char_height = self.font.char_data(b'A').unwrap().height;

        for word in string.split_whitespace() {
            // Work out size of this word
            let (this_char_x, this_char_y) = self.font.string_size(word);

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
                    let (buffer_width, _) = self.font.string_size(&buffer);
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

                x = self.font.string_size(&buffer).0;

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
        (lines, char_height.into(), y + char_height as i16)
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
    fn char_data(&self, c: u8) -> Option<AsciiFontGlyph<'static>>;

    fn string_size(&self, string: &str) -> (i16, i16) {
        // Use 'A' as the height of a line
        let line_height = self.char_data(b'A').unwrap().height;

        let mut current_line_width = 0;
        let mut longest_line_width = 0;
        let mut height = line_height;

        for c in string.chars() {
            if c != '\n' {
                if let Some(glyph) = self.char_data(c as u8) { // TODO: unsafe cast
                    // Update line width
                    current_line_width += glyph.width;
                    if current_line_width > longest_line_width {
                        longest_line_width = current_line_width;
                    }
                }
            } else {
                height += line_height;
                current_line_width = 0;
            }
        }

        (longest_line_width as i16, height as i16)
    }
}

#[derive(Debug)]
pub struct AsciiFontGlyph<'a> {
    pub width: u8,
    pub height: u8,
    pub data: &'a [u8],
}
