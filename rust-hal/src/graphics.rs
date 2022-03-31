use core::convert::Infallible;

use alloc::{vec, vec::Vec};

use crate::util::saturating_into::SaturatingInto;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Colour(pub u16);

impl Colour {
    pub fn high_byte(&self) -> u8 {
        ((self.0 & 0xFF00) >> 8) as u8
    }

    pub fn low_byte(&self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    pub fn as_bytes(&self) -> (u8, u8) {
        (self.high_byte(), self.low_byte())
    }

    pub const BLACK: Colour = Colour(0);
}

pub trait DrawingSurface {
    type Error;

    fn fill_surface(&mut self, colour: Colour) -> Result<(), Self::Error>;
    fn draw_filled_rect(&mut self, x: i16, y: i16, w: u16, h: u16, colour: Colour) -> Result<(), Self::Error>;
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
}

impl DrawingSurface for Sprite {
    type Error = Infallible;

    fn fill_surface(&mut self, colour: Colour) -> Result<(), Self::Error> {
        self.data.fill(colour);
        Ok(())
    }

    fn draw_filled_rect(&mut self, x: i16, y: i16, mut w: u16, h: u16, colour: Colour) -> Result<(), Self::Error> {
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

        // Draw line-by-line
        for curr_y in y..(y as i32 + h as i32).saturating_into() {
            if curr_y < 0 { continue; }
            let curr_y = curr_y as usize;

            self.data[
                (curr_y * self.width as usize + x)
                ..(curr_y * self.width as usize + x + w as usize)
            ].fill(colour);
        }

        Ok(())
    }
}
