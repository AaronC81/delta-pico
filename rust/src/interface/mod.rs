mod display;
use core::{str::from_utf8, slice};

use alloc::string::String;
pub use display::*;

mod buttons;
pub use buttons::*;

mod storage;
pub use storage::*;

mod usb_mass_storage;
pub use usb_mass_storage::*;

static mut FRAMEWORK: *mut ApplicationFrameworkInterface = 0 as *mut _;
pub fn framework() -> &'static mut ApplicationFrameworkInterface {
    unsafe {
        FRAMEWORK.as_mut().unwrap()
    }
}

#[no_mangle]
pub extern "C" fn delta_pico_set_framework(fw: *mut ApplicationFrameworkInterface) {
    unsafe {
        FRAMEWORK = fw;
    }
}

#[repr(C)]
pub struct ApplicationFrameworkInterface {
    pub debug_handler: extern "C" fn(*const u8) -> (),

    pub millis: extern "C" fn() -> u32,
    pub micros: extern "C" fn() -> u32,

    pub charge_status: extern "C" fn() -> i32,
    pub heap_usage: extern "C" fn(*mut u64, *mut u64) -> (),

    hardware_revision: *const u8,

    pub display: DisplayInterface,
    pub buttons: ButtonsInterface,
    pub storage: StorageInterface,
    pub usb_mass_storage: UsbMassStorageInterface,

    // Bit of a hack to have these here... ah well
    pub rbop_location_x: i64,
    pub rbop_location_y: i64,
}

impl ApplicationFrameworkInterface {
    pub fn hardware_revision(&self) -> String {
        // Very dodgy C -> Rust string conversion
        unsafe {
            let mut string_length = 0;
            let mut ptr = self.hardware_revision;

            while *ptr != 0 {
                ptr = ptr.offset(1);
                string_length += 1;
            }

            from_utf8(slice::from_raw_parts(self.hardware_revision, string_length)).unwrap().into()
        }
    }
}
