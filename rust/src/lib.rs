#![no_std]
#![feature(default_alloc_error_handler)]

extern crate alloc;

mod c_allocator;

use core::{panic::PanicInfo, cell::RefCell};
use alloc::{format, string::String, vec::Vec, vec, rc::Rc};
use c_allocator::CAllocator;

mod interface;
mod operating_system;
mod rbop_impl;
mod applications;
mod filesystem;
mod timer;
mod multi_tap;

use fatfs::{FileSystem, FsOptions, Write, IoBase, Read, Seek};
use interface::framework;
use operating_system::os;

use crate::{interface::Colour, operating_system::{OSInput, OperatingSystemInterface}};

#[global_allocator]
static ALLOCATOR: CAllocator = CAllocator;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    framework().display.switch_to_screen();
    framework().display.fill_screen(Colour::BLACK);

    // Draw panic title bar
    framework().display.draw_rect(
        0, 0, framework().display.width as i64, OperatingSystemInterface::TITLE_BAR_HEIGHT,
        Colour::RED, interface::ShapeFill::Filled, 0,
    );
    framework().display.print_at(5, 7, "Panic   :(");

    // Draw error text
    let (lines, line_height, _) =
        framework().display.wrap_text(&format!("{}", info), framework().display.width as i64 - 20);
    for (i, line) in lines.iter().enumerate() {
        framework().display.print_at(
            10, OperatingSystemInterface::TITLE_BAR_HEIGHT + 5 + line_height * i as i64,
            line
        );
    }

    // Draw keys
    framework().display.print_at(
        0, framework().display.height as i64 - 50, "Restart the device, or use\n[EXE] to enter bootloader"
    );
    
    framework().display.draw();

    loop {
        if let Some(OSInput::Exe) = framework().buttons.wait_press() {
            os().reboot_into_bootloader();
        }
    }
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
    os().application_list.add::<applications::graph::GraphApplication>();
    os().application_list.add::<applications::tetris::TetrisApplication>();
    os().application_list.add::<applications::numbers_game::NumbersGame>();
    os().application_list.add::<applications::about::AboutApplication>();
    os().application_list.add::<applications::settings::SettingsApplication>();
    os().application_list.add::<applications::storage::StorageApplication>();
    os().application_list.add::<applications::bootloader::BootloaderApplication>();

    if !(framework().storage.connected)() {
        os().ui_text_dialog("Unable to communicate with storage.");
    }

    loop {
        os().application_to_tick().tick();
    }
}
