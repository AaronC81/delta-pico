mod display;
use core::{str::from_utf8, slice};

use alloc::{string::String, boxed::Box};
pub use display::*;

// mod buttons;
// pub use buttons::*;

// mod storage;
// pub use storage::*;

// mod usb_mass_storage;
// pub use usb_mass_storage::*;

pub trait ApplicationFramework {
    type DisplayI : DisplayInterface;

    fn display(&self) -> &Self::DisplayI;
    fn display_mut(&mut self) -> &mut Self::DisplayI;
    fn hardware_revision(&self) -> String;
}
