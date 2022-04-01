#![no_std]
#![feature(default_alloc_error_handler)]

extern crate alloc;

mod c_allocator;

use core::{panic::PanicInfo};
use alloc::{format, string::String};
use c_allocator::CAllocator;

pub mod interface;
pub mod operating_system;
pub mod rbop_impl;
pub mod applications;
pub mod filesystem;
pub mod timer;
pub mod multi_tap;
pub mod tests;

use interface::framework;
use operating_system::os;

use crate::{interface::Colour, operating_system::{OSInput, OperatingSystemInterface}};

fn debug(info: String) {
    let mut message_bytes = info.as_bytes().to_vec();
    message_bytes.push(0);

    (framework().debug_handler)(message_bytes.as_ptr());
}

#[no_mangle]
pub extern "C" fn delta_pico_main() {
    framework().display.fill_screen(Colour(0xFFFF));
    framework().display.draw();

    debug("Rust main!".into());

    os().application_list.add::<applications::calculator::CalculatorApplication>();
    os().application_list.add::<applications::graph::GraphApplication>();
    os().application_list.add::<applications::numbers_game::NumbersGame>();
    os().application_list.add::<applications::files::FilesApplication>();
    os().application_list.add::<applications::about::AboutApplication>();
    os().application_list.add::<applications::settings::SettingsApplication>();
    os().application_list.add::<applications::storage::StorageApplication>();
    os().application_list.add::<applications::bootloader::BootloaderApplication>();

    if !(framework().storage.connected)() {
        os().ui_text_dialog("Unable to communicate with storage.");
    }

    // Show a splash screen while we load storage
    framework().display.fill_screen(Colour::BLACK);
    framework().display.draw_bitmap(60, 80, "splash");
    framework().display.draw();

    // Temporary
    framework().storage.with_priority(|| {
        // We use `leak` to ensure `fat` doesn't get dropped at the end of this `with_priority` call
        let fat = os().filesystem.fat.read_all().unwrap();
        framework().usb_mass_storage.fat12_filesystem = fat.leak().as_mut_ptr();    
        
        (framework().usb_mass_storage.begin)();
    });

    loop {
        os().application_to_tick().tick();
    }
}
