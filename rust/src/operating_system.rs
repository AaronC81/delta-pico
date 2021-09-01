use alloc::{boxed::Box, format, string::String, vec};
use rbop::{UnstructuredNode, node::unstructured::{UnstructuredNodeRoot, Upgradable}, render::{Area, Renderer, Viewport}};
use rust_decimal::Decimal;
use core::{cmp::max, mem};

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
    /// Replaces the currently-running application with a new instance of the application at `index`
    /// in `application_list`.
    pub fn launch_application(&mut self, index: usize) {
        self.showing_menu = false;
        self.active_application = Some(self.application_list.applications[index].1());
    }

    /// Returns a reference to the application which should be ticked. This is typically the running
    /// application, unless showing the menu, in which case it is the menu application itself.
    pub fn application_to_tick(&mut self) -> &mut dyn Application {
        if self.showing_menu {
            &mut self.menu
        } else {
            self.active_application.as_mut()
                .map(|x| x.as_mut())
                .unwrap_or(&mut self.menu)
        }
    }

    /// Toggles whether the global menu is currently being shown.
    pub fn toggle_menu(&mut self) {
        self.showing_menu = !self.showing_menu;
    }

    /// Reboots the Raspberry Pi Pico into its bootloader. This halts the software and cannot be
    /// exited without a power cycle.
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

    /// Draws a title bar to the top of the screen, with the text `s`.
    pub fn ui_draw_title(&mut self, s: impl Into<String>) {
        (framework().display.draw_rect)(
            0, 0, framework().display.width as i64, 30,
            crate::graphics::colour::ORANGE, true, 0
        );
        (framework().display.set_cursor)(5, 7);
        framework().display.print(s);
    }

    /// Opens a menu with the items in the slice `items`. The user can navigate the menu with the
    /// up and down keys, and select an item with EXE.
    /// Returns Some(the index of the item selected).
    /// These menus are typically to be opened with the LIST key. If `can_close` is true, pressing
    /// LIST will return None.
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

    /// Opens an rbop input box with the given `title` and optionally starts the node tree at the
    /// given `root`. When the user presses EXE, returns the current node tree.
    pub fn ui_input_expression(&mut self, title: impl Into<String>, root: Option<UnstructuredNodeRoot>) -> UnstructuredNodeRoot {
        const PADDING: u64 = 10;
        
        let mut rbop_ctx = RbopContext {
            viewport: Some(Viewport::new(Area::new(
                framework().display.width - PADDING * 2,
                framework().display.height - PADDING * 2,
            ))),
            ..RbopContext::new()
        };

        if let Some(unr) = root {
            rbop_ctx.root = unr;
        }

        let title = title.into();

        // Don't let the box get any shorter than the maximum height it has achieved, or you'll get
        // ghost boxes if the height reduces since we don't redraw the whole frame
        let mut minimum_height = 0;
        
        loop {
            // Calculate layout in advance so we know height
            let layout = framework().layout(
                &rbop_ctx.root,
                Some(&mut rbop_ctx.nav_path.to_navigator()),
            );
            let height = max(layout.area(framework()).height, minimum_height);

            if height > minimum_height {
                minimum_height = height;
            }

            // Draw background
            let y = framework().display.height
                - height
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
                    return rbop_ctx.root;
                } else {
                    rbop_ctx.input(input);
                }
            }
        }
    }

    /// A variant of `ui_input_expression` which upgrades and evaluates the input.
    /// If this causes an error, a dialog will be displayed with `ui_text_dialog`, which will
    /// require redrawing the screen once dismissed. As such, this takes a `redraw` function which
    /// will be called each time before displaying the input prompt (including the first time).
    pub fn ui_input_expression_and_evaluate(
        &mut self,
        title: impl Into<String>,
        root: Option<UnstructuredNodeRoot>,
        mut redraw: impl FnMut(),
    ) -> Decimal {
        let title = title.into();
        let mut unr = root;
        loop {
            redraw();
            unr = Some(os().ui_input_expression(title.clone(), unr));
            match unr
                .as_ref()
                .unwrap()
                .upgrade()
                .map_err(|e| format!("{:?}", e))
                .and_then(|sn| sn
                    .evaluate()
                    .map_err(|e| format!("{:?}", e))) {
                
                Ok(d) => {
                    return d;
                }
                Err(s) => {
                    redraw();
                    os().ui_text_dialog(s);
                }
            }
        }
    }

    /// Opens a text dialog in the centre of the screen which can be dismissed with EXE.
    pub fn ui_text_dialog(&mut self, s: impl Into<String>) {
        const H_PADDING: i64 = 30;
        const H_INNER_PADDING: i64 = 10;
        const V_PADDING: i64 = 10;
        let w = framework().display.width as i64 - H_PADDING * 2;
        let (lines, ch, h) = framework().display.wrap_text(s, w - H_INNER_PADDING * 2);
        let y_start = (framework().display.height as i64 - h) / 2;

        (framework().display.draw_rect)(
            H_PADDING, y_start,
            w, h + V_PADDING * 2,
            colour::GREY, true, 10
        );
        (framework().display.draw_rect)(
            H_PADDING, y_start,
            w, h + V_PADDING * 2,
            colour::WHITE, false, 10
        );
        
        for (i, line) in lines.iter().enumerate() {
            (framework().display.set_cursor)(H_PADDING + H_INNER_PADDING, y_start + V_PADDING + ch * i as i64);
            framework().display.print(line);
        }

        // Push to screen
        (framework().display.draw)();

        // Poll for input
        loop {
            if let Some(input) = framework().buttons.poll_press() {
                if ButtonInput::Exe == input {
                    break;
                }
            }
        }
    }
}
