use core::alloc::{GlobalAlloc, Layout};

pub struct CAllocator;
unsafe impl GlobalAlloc for CAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        extern "C" { fn malloc(size: usize) -> *mut u8; }
        malloc(layout.size())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
        extern "C" { fn free(ptr: *mut u8); }
        free(ptr);
    }
}
