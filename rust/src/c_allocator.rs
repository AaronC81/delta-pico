use core::alloc::{GlobalAlloc, Layout};

pub static mut MEMORY_USAGE: usize = 0;
pub static mut EXTERNAL_MEMORY_USAGE: usize = 0;

pub struct CAllocator;

extern "C" { fn malloc_usable_size(ptr: *mut u8) -> usize; }

unsafe impl GlobalAlloc for CAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        extern "C" { fn malloc(size: usize) -> *mut u8; }
        let ptr = malloc(layout.size());
        MEMORY_USAGE += malloc_usable_size(ptr);
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
        extern "C" { fn free(ptr: *mut u8); }
        MEMORY_USAGE -= malloc_usable_size(ptr);
        free(ptr);
    }
}

impl CAllocator {
    pub fn count_external_alloc(&self, ptr: *mut u8) {
        unsafe {
            EXTERNAL_MEMORY_USAGE += malloc_usable_size(ptr);
        }
    }

    pub fn count_external_free(&self, ptr: *mut u8) {
        unsafe {
            EXTERNAL_MEMORY_USAGE -= malloc_usable_size(ptr);
        }
    }
}
