use alloc::{boxed::Box, vec};
use core::mem;

use crate::applications::{Application, ApplicationList, menu::MenuApplication};

static mut OPERATING_SYSTEM_INTERFACE: Option<OperatingSystemInterface> = None;
pub fn os() -> &'static mut OperatingSystemInterface {
    unsafe {
        if OPERATING_SYSTEM_INTERFACE.is_none() {
            OPERATING_SYSTEM_INTERFACE = Some(OperatingSystemInterface {
                application_list: ApplicationList::new(),
                active_application: None, 
                menu: MenuApplication::new(),
                showing_menu: true,
            });
        }
        OPERATING_SYSTEM_INTERFACE.as_mut().unwrap()
    }
}

pub struct OperatingSystemInterface {
    pub application_list: ApplicationList,
    pub menu: MenuApplication,
    pub showing_menu: bool,
    pub active_application: Option<Box<dyn Application>>,
}

impl OperatingSystemInterface {
    pub fn launch_application(&mut self, index: usize) {
        self.showing_menu = false;
        self.active_application = Some(self.application_list.applications[index].1());
    }

    pub fn application_to_tick(&mut self) -> &mut dyn Application {
        if self.showing_menu {
            &mut self.menu
        } else {
            self.active_application.as_mut()
                .map(|x| x.as_mut())
                .unwrap_or(&mut self.menu)
        }
    }

    pub fn toggle_menu(&mut self) {
        self.showing_menu = !self.showing_menu;
    }

    pub fn reboot_into_bootloader(&mut self) -> ! {
        // Awww, yeah!
        // This is a translation of the parts of...
        //   - https://github.com/raspberrypi/pico-sdk/blob/master/src/rp2_common/pico_bootrom/bootrom.c
        //   - https://github.com/raspberrypi/pico-sdk/blob/master/src/rp2_common/pico_bootrom/include/pico/bootrom.h
        // ...required to call `reset_usb_boot`.
        // Nothing super fancy is going on here, just lots of casting pointers around.
        // The mem::transmute calls are required because Rust doesn't allow you to cast `*const _`
        // to `extern "C" fn(...) -> _`, even though the latter is still just a pointer in memory.
        unsafe {
            // Resolve a function which allows us to look up items in ROM tables
            let rom_table_lookup_fn_addr = *(0x18 as *const u16) as *const ();
            let rom_table_lookup_fn: extern "C" fn(*const u16, u32) -> *const () = mem::transmute(rom_table_lookup_fn_addr);
            
            // Use that function to look up the address of the USB bootloader function
            let usb_boot_fn_code = (('B' as u32) << 8) | ('U' as u32);
            let func_table = *(0x14 as *const u16) as *const u16;
            let usb_boot_fn_addr = rom_table_lookup_fn(func_table, usb_boot_fn_code);

            // Call that function
            let usb_boot_fn: extern "C" fn(u32, u32) = mem::transmute(usb_boot_fn_addr);
            usb_boot_fn(0, 0);
        }
        panic!("failed to access bootloader")
    }
}
