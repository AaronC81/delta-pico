#![no_std]
#![feature(default_alloc_error_handler)]

extern crate alloc;

mod c_allocator;

use core::panic::PanicInfo;
use alloc::{format, string::{String}, vec::Vec};
use applications::{Application, ApplicationList};
use c_allocator::CAllocator;

mod interface;
mod operating_system;
mod rbop_impl;
mod applications;
mod graphics;

use interface::framework;
use operating_system::os;

#[global_allocator]
static ALLOCATOR: CAllocator = CAllocator;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let message = format!("{}", info);
    let mut message_bytes = message.as_bytes().iter().cloned().collect::<Vec<_>>();
    message_bytes.push(0);

    (framework().panic_handler)(message_bytes.as_ptr());
    loop {}
}

fn debug(info: String) {
    let mut message_bytes = info.as_bytes().iter().cloned().collect::<Vec<_>>();
    message_bytes.push(0);

    (framework().debug_handler)(message_bytes.as_ptr());
}

#[no_mangle]
pub extern "C" fn delta_pico_main() {
    debug("Rust main!".into());

    os().application_list.add::<applications::calculator::CalculatorApplication>();
    os().application_list.add::<applications::about::AboutApplication>();
    os().application_list.add::<applications::bootloader::BootloaderApplication>();

    loop {
        os().application_to_tick().tick();
    }
}