use alloc::{boxed::Box, string::String, vec};
use rbop::{UnstructuredNode, node::unstructured::UnstructuredNodeRoot, render::{Area, Renderer, Viewport}};
use core::mem;

use crate::{applications::{Application, ApplicationList, menu::MenuApplication}, graphics::colour, interface::{ButtonInput, framework}, rbop_impl::RbopContext};

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

    pub fn ui_draw_title(&mut self, s: impl Into<String>) {
        (framework().display.draw_rect)(
            0, 0, framework().display.width as i64, 30,
            crate::graphics::colour::ORANGE, true, 0
        );
        (framework().display.set_cursor)(5, 7);
        framework().display.print(s);
    }

    pub fn ui_open_menu(&mut self, items: &[String], can_close: bool) -> Option<usize> {
        const ITEM_GAP: i64 = 30;
        let mut selected_index = 0;

        loop {
            // Draw background
            let mut y = (framework().display.height as i64 - ITEM_GAP * items.len() as i64 - 10) as i64;
            (framework().display.draw_rect)(0, y, 240, 400, colour::GREY, true, 10);
            (framework().display.draw_rect)(0, y, 240, 400, colour::WHITE, false, 10);

            // Draw items
            y += 10;
            for (i, item) in items.iter().enumerate() {
                if i == selected_index {
                    (framework().display.draw_rect)(
                        5, y, framework().display.width as i64 - 5 * 2, 25,
                        crate::graphics::colour::BLUE, true, 7
                    );
                }
                (framework().display.set_cursor)(10, y as i64 + 4);
                framework().display.print(item);

                y += ITEM_GAP;
            }

            (framework().display.draw)();

            if let Some(btn) = framework().buttons.poll_press() {
                match btn {
                    ButtonInput::MoveUp => {
                        if selected_index == 0 {
                            selected_index = items.len() - 1;
                        } else {
                            selected_index -= 1;
                        }
                    }
                    ButtonInput::MoveDown => {
                        selected_index += 1;
                        selected_index %= items.len();
                    }
                    ButtonInput::Exe => return Some(selected_index),
                    ButtonInput::List if can_close => return None,
                    _ => (),
                }
            }
        }
    }

    pub fn ui_input_expression<R>(
        &mut self,
        title: impl Into<String>,
        transformer: impl Fn(UnstructuredNodeRoot) -> Result<R, String>,
    ) -> R {
        const PADDING: u64 = 10;
        
        let mut rbop_ctx = RbopContext {
            viewport: Some(Viewport::new(Area::new(
                framework().display.width - PADDING * 2,
                framework().display.height - PADDING * 2,
            ))),
            ..RbopContext::new()
        };

        let title = title.into();
        
        loop {
            // Calculate layout in advance so we know height
            let layout = framework().layout(
                &rbop_ctx.root,
                Some(&mut rbop_ctx.nav_path.to_navigator()),
            );

            // Draw background
            let y = framework().display.height
                - layout.area(framework()).height
                - 30
                - PADDING * 2;
            (framework().display.draw_rect)(0, y as i64, 240, 400, colour::GREY, true, 10);
            (framework().display.draw_rect)(0, y as i64, 240, 400, colour::WHITE, false, 10);      
            
            // Draw title
            (framework().display.set_cursor)(PADDING as i64, (y + PADDING) as i64);
            framework().display.print(title.clone());

            // Draw expression
            framework().rbop_location_x = PADDING;
            framework().rbop_location_y = y + 30 + PADDING;
            framework().draw_all(
                &rbop_ctx.root, 
                Some(&mut rbop_ctx.nav_path.to_navigator()),
                rbop_ctx.viewport.as_ref(),
            );

            // Push to screen
            (framework().display.draw)();

            // Poll for input
            if let Some(input) = framework().buttons.poll_press() {
                if ButtonInput::Exe == input {
                    match transformer(rbop_ctx.root) {
                        Ok(result) => return result,
                        Err(_) => todo!() // TODO: error dialog
                    }
                } else {
                    rbop_ctx.input(input);
                }
            }
        }
    }
}
