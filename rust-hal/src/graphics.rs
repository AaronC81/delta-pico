use core::convert::Infallible;

use alloc::{vec, vec::Vec};
use delta_pico_rust::interface::Colour;

use crate::util::saturating_into::SaturatingInto;

pub trait DrawingSurface {
    type Error;

    fn fill_surface(&mut self, colour: Colour) -> Result<(), Self::Error>;
    fn draw_rect(&mut self, x: i16, y: i16, w: u16, h: u16, filled: bool, colour: Colour) -> Result<(), Self::Error>;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Sprite {
    pub width: u16,
    pub height: u16,
    pub data: Vec<Colour>,
}

impl Sprite {
    pub fn new(width: u16, height: u16) -> Sprite {
        Sprite {
            width,
            height,
            data: vec![Colour::BLACK; width as usize * height as usize],
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
}

impl DrawingSurface for Sprite {
    type Error = Infallible;

    fn fill_surface(&mut self, colour: Colour) -> Result<(), Self::Error> {
        self.data.fill(colour);
        Ok(())
    }

    fn draw_rect(&mut self, x: i16, y: i16, mut w: u16, mut h: u16, filled: bool, colour: Colour) -> Result<(), Self::Error> {
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

        Ok(())
    }
}
