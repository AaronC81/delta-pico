#![no_std]
#![feature(default_alloc_error_handler)]

extern crate alloc;

mod c_allocator;

use core::{panic::PanicInfo, cell::RefCell};
use alloc::{format, string::String, rc::Rc, boxed::Box};
use applications::{about::{AboutApplication, self}, Application};
use c_allocator::CAllocator;

pub mod interface;
pub mod operating_system;
// pub mod rbop_impl;
pub mod applications;
// pub mod filesystem;
pub mod timer;
pub mod multi_tap;
// pub mod tests;

use interface::{ApplicationFramework, DisplayInterface, ButtonInput, StorageInterface};

use crate::{interface::Colour, operating_system::{OSInput, OperatingSystem}};

static mut PANIC_OS_POINTER: *mut () = core::ptr::null_mut();
static mut PANIC_HANDLER: Option<Box<dyn FnMut(&PanicInfo) -> ()>> = None;

pub extern "C" fn delta_pico_main<F: ApplicationFramework + 'static>(framework: F) {
    let mut os = OperatingSystem::new(framework);

    os.framework.display_mut().fill_screen(Colour(0xFFFF));
    os.framework.display_mut().draw();

    os.application_list.os = &mut os as *mut _;

    // os().application_list.add::<applications::calculator::CalculatorApplication>();
    // os().application_list.add::<applications::graph::GraphApplication>();
    os.application_list.add::<applications::numbers_game::NumbersGame<F>>();
    // os().application_list.add::<applications::files::FilesApplication>();
    os.application_list.add::<applications::about::AboutApplication<F>>();
    os.application_list.add::<applications::settings::SettingsApplication<F>>();
    os.application_list.add::<applications::storage::StorageApplication<F>>();
    os.application_list.add::<applications::bootloader::BootloaderApplication<F>>();

    if !os.framework.storage_mut().is_connected() {
        os.ui_text_dialog("Unable to communicate with storage.");
    }

    // Show a splash screen while we load storage
    os.framework.display_mut().fill_screen(Colour::BLACK);
    os.framework.display_mut().draw_bitmap(60, 80, "splash");
    os.framework.display_mut().draw();

    // Temporary
    // framework().storage.with_priority(|| {
    //     // We use `leak` to ensure `fat` doesn't get dropped at the end of this `with_priority` call
    //     let fat = os().filesystem.fat.read_all().unwrap();
    //     framework().usb_mass_storage.fat12_filesystem = fat.leak().as_mut_ptr();    
        
    //     (framework().usb_mass_storage.begin)();
    // });

    // Set up menu
    os.menu = Some(applications::menu::MenuApplication::new(&mut os as *mut _));

    // Set up a panic handler!
    // Yeah, this is super unsafe, but we can't use `panic_handler` because we don't know the T in 
    // `OperatingSystem<T>`.
    unsafe {
        PANIC_OS_POINTER = &mut os as *mut OperatingSystem<F> as *mut ();
        PANIC_HANDLER = Some(Box::new(|info| {
            let os = (PANIC_OS_POINTER as *mut OperatingSystem<F>).as_mut().unwrap();

            os.framework.display_mut().switch_to_screen();
            os.framework.display_mut().fill_screen(Colour::BLACK);
        
            // Draw panic title bar
            let width = os.framework.display().width();
            os.framework.display_mut().draw_rect(
                0, 0, width, 30,
                Colour::RED, interface::ShapeFill::Filled, 0,
            );
            os.framework.display_mut().print_at(5, 7, "Panic   :(");
        
            // Draw error text
            let (lines, line_height, _) =
                os.framework.display_mut().wrap_text(&format!("{}", info), width - 20);
            for (i, line) in lines.iter().enumerate() {
                os.framework.display_mut().print_at(
                    10, 30 + 5 + line_height * i as i16,
                    line
                );
            }
        
            // Draw keys
            let height = os.framework.display().height();
            os.framework.display_mut().print_at(
                0, height as i16 - 50, "Restart the device, or use\n[EXE] to enter bootloader"
            );
            
            os.framework.display_mut().draw();
        
            loop {
                if let Some(OSInput::Button(ButtonInput::Exe)) = os.input() {
                    os.framework.reboot_into_bootloader();
                }
            }
        }));
    }

    loop {
        os.application_to_tick().tick();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        PANIC_HANDLER.as_mut().unwrap()(info);

        loop {}
    }
}
