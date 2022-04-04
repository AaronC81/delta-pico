use core::fmt::Debug;

use alloc::{string::{String, ToString}, vec, vec::Vec};

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
