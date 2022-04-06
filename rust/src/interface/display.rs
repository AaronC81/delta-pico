use core::fmt::Debug;

use crate::graphics::Sprite;

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

    /// Breaks this colour into its individual RGB565 components: (red, green, blue).
    pub fn to_parts(self) -> (u16, u16, u16) {
        (
            (self.0 & 0b1111100000000000) >> 11,
            (self.0 & 0b0000011111100000) >> 5,
            self.0 & 0b0000000000011111,
        )
    }

    /// Creates a colour from RGB565 components.
    ///
    /// For speed, this method does not bounds-check the arguments; red/blue components over 31, or
    /// a green component over 63, will cause interference between the colour channels. It is the
    /// caller's responsibility to ensure these are bounded.
    pub fn from_parts(red: u16, green: u16, blue: u16) -> Self {
        Self(
            ((red) << 11)
            | ((green) << 5)
            | blue
        )
    }

    /// Given another colour, interpolates between the two colours based on the amount given.
    /// **Only the lowest four bits of the `amount` are considered**.
    /// 
    /// If the amount is 0, the self colour is returned.
    /// If it is 0xF (or greater), the other colour is returned.
    /// For values in between these, a linear interpolation between the two colours is calculated.
    pub fn interpolate_with_nibble(self, other: Colour, amount: u8) -> Colour {
        let (self_r, self_g, self_b) = self.to_parts();
        let (other_r, other_g, other_b) = other.to_parts();

        Colour::from_parts(
            self_r + (amount as u16 * (other_r - self_r)) / 16,
            self_g + (amount as u16 * (other_g - self_g)) / 16,
            self_b + (amount as u16 * (other_b - self_b)) / 16,
        )
    }
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
    fn width(&self) -> u16;
    fn height(&self) -> u16;

    fn draw_display_sprite(&mut self, sprite: &Sprite);
}
