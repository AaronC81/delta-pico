#[repr(C)]
pub struct UsbMassStorageInterface {
    pub fat12_filesystem: *mut u8,
    pub active: bool,
    pub enter: extern "C" fn() -> bool,
}
