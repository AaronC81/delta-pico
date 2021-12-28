use alloc::{vec, vec::Vec};

#[repr(C)]
pub struct StorageInterface {
    pub connected: extern "C" fn() -> bool,
    pub busy: extern "C" fn() -> bool,
    pub write: extern "C" fn(address: u16, count: u16, buffer: *const u8) -> bool,
    pub read: extern "C" fn(address: u16, count: u16, buffer: *mut u8) -> bool,

    pub acquire_priority: extern "C" fn() -> (),
    pub release_priority: extern "C" fn() -> (),
}

impl StorageInterface {
    pub fn read(&self, address: u16, count: u16) -> Option<Vec<u8>> {
        let mut buffer = vec![0; count as usize];
        if (self.read)(address, count, buffer.as_mut_ptr()) {
            Some(buffer)
        } else {
            None
        }
    }

    pub fn write(&self, address: u16, bytes: &[u8]) -> Option<()> {
        if (self.write)(address, bytes.len() as u16, bytes.as_ptr()) {
            Some(())
        } else {
            None
        }
    }

    pub fn acquire_priority(&self) {
        (self.acquire_priority)();
    }

    pub fn release_priority(&self) {
        (self.release_priority)();
    }

    pub fn with_priority<T, F>(&self, func: F) -> T where F : FnOnce() -> T {
        self.acquire_priority();
        let result = func();
        self.release_priority();
        result
    }

    pub fn clear_range(&self, start: u16, length: u16) -> Option<()> {
        const CHUNK_SIZE: u8 = 64;
        let buffer = [0; CHUNK_SIZE as usize];

        let mut bytes_remaining = length;
        let mut address = start;
        while bytes_remaining > 0 {
            let this_chunk_size = core::cmp::min(CHUNK_SIZE as u16, bytes_remaining);
            if !(self.write)(address, this_chunk_size as u16, buffer.as_ptr()) {
                return None;
            }

            address += this_chunk_size;
            bytes_remaining -= this_chunk_size;
        }

        Some(())
    }
}
