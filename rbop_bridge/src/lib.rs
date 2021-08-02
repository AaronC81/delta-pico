#![no_std]
#![feature(default_alloc_error_handler)]

extern crate alloc;

mod c_allocator;

use core::panic::PanicInfo;
use alloc::{vec, boxed::Box};
use rbop::{UnstructuredNodeList, nav::NavPath, node::unstructured::UnstructuredNodeRoot};
use c_allocator::CAllocator;

#[global_allocator]
static ALLOCATOR: CAllocator = CAllocator;

pub struct RbopContext {
    root: UnstructuredNodeRoot,
    nav_path: NavPath,
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rbop_new() -> *mut RbopContext {
    Box::into_raw(Box::new(RbopContext {
        root: UnstructuredNodeRoot { root: UnstructuredNodeList { items: vec![] } },
        nav_path: NavPath::new(vec![]),
    }))
}
