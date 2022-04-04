use core::convert::Infallible;

use alloc::{vec, vec::Vec};
use crate::interface::Colour;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Sprite {
    pub width: u16,
    pub height: u16,
    pub data: Vec<Colour>,

    pub cursor_x: i16,
    pub cursor_y: i16,
}

impl Sprite {
    pub fn new(width: u16, height: u16) -> Sprite {
        Sprite {
            width,
            height,
            data: vec![Colour::BLACK; width as usize * height as usize],
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    pub fn pixel(&mut self, x: u16, y: u16) -> &mut Colour {
        &mut self.data[y as usize * self.width as usize + x as usize]
    }

    pub fn try_pixel(&mut self, x: i16, y: i16) -> Option<&mut Colour> {
        if y as usize * self.width as usize + x as usize > self.data.len() {
            None
        } else {
            Some(self.pixel(x as u16, y as u16))
        }
    }

    fn fill_surface(&mut self, colour: Colour) {
        self.data.fill(colour);
    }

    fn draw_rect(&mut self, x: i16, y: i16, mut w: u16, mut h: u16, filled: bool, colour: Colour) {
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

            if filled {
                self.data[
                    (curr_y * self.width as usize + x)
                    ..(curr_y * self.width as usize + x + w as usize)
                ].fill(colour);
            } else {
                // Only draw on a border
                for curr_x in x..(x + w as usize) {
                    if curr_x == x || curr_y == y || curr_x == (x + w as usize - 1) || curr_y == (y + h as usize - 1) {
                        *self.pixel(curr_x as u16, curr_y as u16) = colour;
                    }
                }
            }
        }
    }

    fn draw_char(&mut self, character: u8) {
        let (x, y) = self.get_cursor();

        if character == '\n' as u8 {
            self.cursor_x = 0;
            self.cursor_y += crate::font_data::droid_sans_20::droid_sans_20_lookup('A' as u8).unwrap()[1] as i16;
            return;
        }

        let character_bitmap = crate::font_data::droid_sans_20::droid_sans_20_lookup(character);
        if character_bitmap.is_none() { return; }
        let character_bitmap = character_bitmap.unwrap();

        // TODO: anti-aliasing or any transparency
        // TODO: font colour

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

                if alpha_nibble > 0x8 {
                    if let Some(px) = self.try_pixel(x + ox as i16, y + oy as i16) {
                        *px = Colour(0xFFFF);
                    }
                }
            }
        }

        self.cursor_x += Into::<i16>::into(character_bitmap[0]) - 1;
    } 

    fn draw_bitmap(&mut self, x: i16, y: i16, name: &str) {
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
                            if let Some(px) = self.try_pixel(x + ox as i16, y + oy as i16 + i as i16) {
                                *px = Colour(colour).into();
                            }
                        }
                    }

                    oy += times - 1;
                    index += 3;
                } else {
                    let colour = bitmap[index];
                    if colour != transparency {
                        if let Some(px) = self.try_pixel(x + ox as i16, y + oy as i16) {
                            *px = Colour(colour).into();
                        }
                    }
                    index += 1;
                }

                oy += 1;
            }

            ox += 1;
        }
    }
    fn print(&mut self, s: &str) {
        for c in s.as_bytes() {
            self.draw_char(*c);
        }
    }

    fn set_cursor(&mut self, x: i16, y: i16) {
        self.cursor_x = x;
        self.cursor_y = y;
    }
    fn get_cursor(&self) -> (i16, i16) {
        (self.cursor_x, self.cursor_y)
    }
}
