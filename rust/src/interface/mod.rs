mod display;
pub use display::*;

mod buttons;
pub use buttons::*;

mod storage;
pub use storage::*;

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
    pub panic_handler: extern "C" fn(*const u8) -> (),
    pub debug_handler: extern "C" fn(*const u8) -> (),

    pub millis: extern "C" fn() -> u32,
    pub micros: extern "C" fn() -> u32,

    pub charge_status: extern "C" fn() -> i32,
    pub heap_usage: extern "C" fn(*mut u64, *mut u64) -> (),

    pub display: DisplayInterface,
    pub buttons: ButtonsInterface,
    pub storage: StorageInterface,

    // Bit of a hack to have these here... ah well
    pub rbop_location_x: i64,
    pub rbop_location_y: i64,
}

