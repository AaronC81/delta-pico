mod display;
use core::{str::from_utf8, slice};

use alloc::{string::String, boxed::Box};
pub use display::*;

mod buttons;
pub use buttons::*;

// mod storage;
// pub use storage::*;

// mod usb_mass_storage;
// pub use usb_mass_storage::*;

pub trait ApplicationFramework {
    type DisplayI : DisplayInterface;
    type ButtonsI : ButtonsInterface;

    fn display(&self) -> &Self::DisplayI;
    fn display_mut(&mut self) -> &mut Self::DisplayI;

    fn buttons(&self) -> &Self::ButtonsI;
    fn buttons_mut(&mut self) -> &mut Self::ButtonsI;

    fn hardware_revision(&self) -> String;
    fn reboot_into_bootloader(&mut self) -> !;

    fn millis(&self) -> u64;
    fn micros(&self) -> u64;
}
