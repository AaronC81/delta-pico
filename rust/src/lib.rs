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

use interface::{ApplicationFramework, DisplayInterface};

use crate::{interface::Colour, operating_system::{OSInput, OperatingSystem}};

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
    // os().application_list.add::<applications::settings::SettingsApplication>();
    // os().application_list.add::<applications::storage::StorageApplication>();
    os.application_list.add::<applications::bootloader::BootloaderApplication<F>>();

    // if !(os.framework.storage.connected)() {
    //     os.ui_text_dialog("Unable to communicate with storage.");
    // }

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

    loop {
        os.application_to_tick().tick();
    }
}
