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
    fn draw_filled_rect(&mut self, x: i32, y: i32, w: u32, h: u32, colour: Colour) -> Result<(), Self::Error>;
}
