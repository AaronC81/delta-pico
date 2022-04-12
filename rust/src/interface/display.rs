use core::fmt::Debug;

use crate::graphics::Sprite;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Colour(pub u8);

impl Colour {
    pub const WHITE: Self = Self::from_rgb565(0xFFFF);
    pub const BLACK: Self = Self::from_rgb565(0x0000);
    pub const ORANGE: Self = Self::from_rgb565(0xD340);
    pub const BLUE: Self = Self::from_rgb565(0x0392);
    pub const DARK_BLUE: Self = Self::from_rgb565(0x024B);
    pub const GREY: Self = Self::from_rgb565(0x31A6);
    pub const RED: Self = Self::from_rgb565(0xF800);

    /// Breaks this colour into its individual RGB332 components: (red, green, blue).
    pub fn to_parts(self) -> (u8, u8, u8) {
        (
            (self.0 & 0b11100000) >> 5,
            (self.0 & 0b00011100) >> 2,
            self.0 & 0b00000011,
        )
    }

    /// Creates a colour from RGB332 components.
    ///
    /// For speed, this method does not bounds-check the arguments; red/green components over 7, or
    /// a blue component over 3, will cause interference between the colour channels. It is the
    /// caller's responsibility to ensure these are bounded.
    pub const fn from_parts(red: u8, green: u8, blue: u8) -> Self {
        Self(
            ((red) << 5)
            | ((green) << 2)
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
        // Interpolation is far too imprecise at RGB332 - temporarily convert to 565
        let (self_r, self_g, self_b) = self.to_rgb565_parts();
        let (other_r, other_g, other_b) = other.to_rgb565_parts();

        let interpolated565 = 
            (self_r + (amount as u16 * (other_r - self_r)) / 16) << 11
            | (self_g + (amount as u16 * (other_g - self_g)) / 16) << 5
            | (self_b + (amount as u16 * (other_b - self_b)) / 16);
        Self::from_rgb565(interpolated565)
    }

    pub const fn from_rgb565(c: u16) -> Self {
        Colour::from_parts(
            ((c & 0b1110000000000000) >> 13) as u8, 
            ((c & 0b0000011100000000) >> 8) as u8,
            ((c & 0b0000000000011000) >> 3) as u8,
        )
    }

    // From: https://blog.frankvh.com/2015/03/29/fast-rgb332-to-rgb565-colorspace-conversion/
    pub const RGB332_TO_RGB565_LOOKUP_TABLE: [u16; 256] = [
        0x0000, 0x000a, 0x0015, 0x001f, 0x0120, 0x012a, 0x0135, 0x013f, 
        0x0240, 0x024a, 0x0255, 0x025f, 0x0360, 0x036a, 0x0375, 0x037f, 
        0x0480, 0x048a, 0x0495, 0x049f, 0x05a0, 0x05aa, 0x05b5, 0x05bf, 
        0x06c0, 0x06ca, 0x06d5, 0x06df, 0x07e0, 0x07ea, 0x07f5, 0x07ff, 
        0x2000, 0x200a, 0x2015, 0x201f, 0x2120, 0x212a, 0x2135, 0x213f, 
        0x2240, 0x224a, 0x2255, 0x225f, 0x2360, 0x236a, 0x2375, 0x237f, 
        0x2480, 0x248a, 0x2495, 0x249f, 0x25a0, 0x25aa, 0x25b5, 0x25bf, 
        0x26c0, 0x26ca, 0x26d5, 0x26df, 0x27e0, 0x27ea, 0x27f5, 0x27ff, 
        0x4800, 0x480a, 0x4815, 0x481f, 0x4920, 0x492a, 0x4935, 0x493f, 
        0x4a40, 0x4a4a, 0x4a55, 0x4a5f, 0x4b60, 0x4b6a, 0x4b75, 0x4b7f, 
        0x4c80, 0x4c8a, 0x4c95, 0x4c9f, 0x4da0, 0x4daa, 0x4db5, 0x4dbf, 
        0x4ec0, 0x4eca, 0x4ed5, 0x4edf, 0x4fe0, 0x4fea, 0x4ff5, 0x4fff, 
        0x6800, 0x680a, 0x6815, 0x681f, 0x6920, 0x692a, 0x6935, 0x693f, 
        0x6a40, 0x6a4a, 0x6a55, 0x6a5f, 0x6b60, 0x6b6a, 0x6b75, 0x6b7f, 
        0x6c80, 0x6c8a, 0x6c95, 0x6c9f, 0x6da0, 0x6daa, 0x6db5, 0x6dbf, 
        0x6ec0, 0x6eca, 0x6ed5, 0x6edf, 0x6fe0, 0x6fea, 0x6ff5, 0x6fff, 
        0x9000, 0x900a, 0x9015, 0x901f, 0x9120, 0x912a, 0x9135, 0x913f, 
        0x9240, 0x924a, 0x9255, 0x925f, 0x9360, 0x936a, 0x9375, 0x937f, 
        0x9480, 0x948a, 0x9495, 0x949f, 0x95a0, 0x95aa, 0x95b5, 0x95bf, 
        0x96c0, 0x96ca, 0x96d5, 0x96df, 0x97e0, 0x97ea, 0x97f5, 0x97ff, 
        0xb000, 0xb00a, 0xb015, 0xb01f, 0xb120, 0xb12a, 0xb135, 0xb13f, 
        0xb240, 0xb24a, 0xb255, 0xb25f, 0xb360, 0xb36a, 0xb375, 0xb37f, 
        0xb480, 0xb48a, 0xb495, 0xb49f, 0xb5a0, 0xb5aa, 0xb5b5, 0xb5bf, 
        0xb6c0, 0xb6ca, 0xb6d5, 0xb6df, 0xb7e0, 0xb7ea, 0xb7f5, 0xb7ff, 
        0xd800, 0xd80a, 0xd815, 0xd81f, 0xd920, 0xd92a, 0xd935, 0xd93f, 
        0xda40, 0xda4a, 0xda55, 0xda5f, 0xdb60, 0xdb6a, 0xdb75, 0xdb7f, 
        0xdc80, 0xdc8a, 0xdc95, 0xdc9f, 0xdda0, 0xddaa, 0xddb5, 0xddbf, 
        0xdec0, 0xdeca, 0xded5, 0xdedf, 0xdfe0, 0xdfea, 0xdff5, 0xdfff, 
        0xf800, 0xf80a, 0xf815, 0xf81f, 0xf920, 0xf92a, 0xf935, 0xf93f, 
        0xfa40, 0xfa4a, 0xfa55, 0xfa5f, 0xfb60, 0xfb6a, 0xfb75, 0xfb7f, 
        0xfc80, 0xfc8a, 0xfc95, 0xfc9f, 0xfda0, 0xfdaa, 0xfdb5, 0xfdbf, 
        0xfec0, 0xfeca, 0xfed5, 0xfedf, 0xffe0, 0xffea, 0xfff5, 0xffff 
    ];

    pub fn to_rgb565(&self) -> u16 {
        Self::RGB332_TO_RGB565_LOOKUP_TABLE[self.0 as usize]
    }

    pub fn to_rgb565_parts(self) -> (u16, u16, u16) {
        let rgb565 = self.to_rgb565();
        (
            (rgb565 & 0b1111100000000000) >> 11,
            (rgb565 & 0b0000011111100000) >> 5,
            rgb565 & 0b0000000000011111,
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
