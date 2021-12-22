#[repr(C)]
pub struct UsbMassStorageInterface {
    pub block_num: usize,
    pub block_size: usize,
    pub fat12_filesystem: *mut u8,

    pub active: bool,
    pub enter: extern "C" fn() -> bool,
}
