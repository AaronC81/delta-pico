use alloc::{boxed::Box, format, string::String};
use rbop::{Number, node::unstructured::{UnstructuredNodeRoot, Upgradable}, render::{Area, Renderer, Viewport}};
use core::{cmp::max, mem};

use crate::{applications::{Application, ApplicationList, menu::MenuApplication}, filesystem::{CalculationHistory, ChunkTable, Filesystem}, interface::{Colour, ShapeFill, framework}, multi_tap::MultiTapState, rbop_impl::RbopContext};

static mut OPERATING_SYSTEM_INTERFACE: Option<OperatingSystemInterface> = None;
pub fn os() -> &'static mut OperatingSystemInterface<'static> {
    unsafe {
        if OPERATING_SYSTEM_INTERFACE.is_none() {
            OPERATING_SYSTEM_INTERFACE = Some(OperatingSystemInterface {
                application_list: ApplicationList::new(),
                active_application: None,
                active_application_index: None,
                menu: MenuApplication::new(),
                showing_menu: true,
                filesystem: Filesystem {
                    calculations: CalculationHistory {
                        table: ChunkTable {
                            start_address: 0x1000,
                            chunks: 1024,
                            storage: &mut framework().storage
                        }
                    }
                },
                last_title_millis: 0,
                text_mode: false,
                multi_tap: MultiTapState::new(),
            });
        }
        OPERATING_SYSTEM_INTERFACE.as_mut().unwrap()
    }
}

pub struct OperatingSystemInterface<'a> {
    pub application_list: ApplicationList,
    pub menu: MenuApplication,
    pub showing_menu: bool,

    pub active_application: Option<Box<dyn Application>>,
    pub active_application_index: Option<usize>,

    pub filesystem: Filesystem<'a>,
    pub last_title_millis: u32,

    pub text_mode: bool,
    pub multi_tap: MultiTapState,
}

impl<'a> OperatingSystemInterface<'a> {
    pub const TITLE_BAR_HEIGHT: i64 = 30;

    /// Replaces the currently-running application with a new instance of the application at `index`
    /// in `application_list`.
    pub fn launch_application(&mut self, index: usize) {
        self.showing_menu = false;

        // Why do we need to do this, rather than letting applications just implement `Drop` if they
        // deal with raw pointers to memory?
        // Well, some applications (namely Calculator) can consume a pretty large amount of memory.
        // If you launch such an application, go to menu, and launch it again, the `Drop` is only
        // called _after_ we've constructed a new application.
        // There might not be enough memory to construct something new, so we OOM!
        // Our `destroy` method gives applications an opportunity to clean up before constructing a 
        // new one.
        // I also couldn't get the borrow checker to be satisfied with me passing `drop` a mutable
        // reference.
        if let Some(app) = self.active_application.as_mut() {
            app.destroy();
        }
        self.active_application_index = Some(index);
        self.active_application = Some(self.application_list.applications[index].1());
    }

    /// Restarts the current application. If none is open, panics.
    pub fn restart_application(&mut self) {
        if let Some(index) = self.active_application_index {
            self.launch_application(index);
        } else {
            panic!("no application running to restart");
        }
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
    pub fn ui_draw_title(&mut self, _s: impl Into<String>) {
        let now_millis = (framework().millis)();
        let _millis_elapsed = now_millis - self.last_title_millis;
        self.last_title_millis = now_millis;

        framework().display.draw_rect(
            0, 0, framework().display.width as i64, Self::TITLE_BAR_HEIGHT,
            Colour::ORANGE, ShapeFill::Filled, 0
        );
        // framework().display.print_at(5, 7, format!("{} ({} ms)", s.into(), millis_elapsed));
        let mut used_memory: u64 = 0;
        let mut available_memory: u64 = 0;
        (framework().heap_usage)(&mut used_memory, &mut available_memory);
        used_memory /= 1000;
        available_memory /= 1000;

        framework().display.print(&format!("{}/{}kB", used_memory, available_memory));

        // Draw charge indicator
        let charge_status = (framework().charge_status)();
        let charge_bitmap = if charge_status == -1 { "power_usb".into() } else { format!("battery_{}", charge_status) };
        framework().display.draw_bitmap(200, 6, &charge_bitmap);

        // Draw text indicator
        if os().text_mode {
            framework().display.draw_rect(145, 4, 50, 24, Colour::WHITE, ShapeFill::Hollow, 5);
            if os().multi_tap.shift {
                framework().display.print_at(149, 6, "TEXT");
            } else {
                framework().display.print_at(153, 6, "text");
            }
        }
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
            framework().display.draw_rect(0, y, 240, 400, Colour::GREY, ShapeFill::Filled, 10);
            framework().display.draw_rect(0, y, 240, 400, Colour::WHITE, ShapeFill::Hollow, 10);

            // Draw items
            y += 10;
            for (i, item) in items.iter().enumerate() {
                if i == selected_index {
                    framework().display.draw_rect(
                        5, y, framework().display.width as i64 - 5 * 2, 25,
                        Colour::BLUE, ShapeFill::Filled, 7
                    );
                }
                framework().display.print_at(10, y as i64 + 4, item);

                y += ITEM_GAP;
            }

            framework().display.draw();

            if let Some(btn) = framework().buttons.wait_press() {
                match btn {
                    OSInput::MoveUp => {
                        if selected_index == 0 {
                            selected_index = items.len() - 1;
                        } else {
                            selected_index -= 1;
                        }
                    }
                    OSInput::MoveDown => {
                        selected_index += 1;
                        selected_index %= items.len();
                    }
                    OSInput::Exe => return Some(selected_index),
                    OSInput::List if can_close => return None,
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
            let height = max(layout.area.height, minimum_height);

            if height > minimum_height {
                minimum_height = height;
            }

            // Draw background
            let y = framework().display.height
                - height
                - 30
                - PADDING * 2;
            framework().display.draw_rect(0, y as i64, 240, 400, Colour::GREY, ShapeFill::Filled, 10);
            framework().display.draw_rect(0, y as i64, 240, 400, Colour::WHITE, ShapeFill::Hollow, 10);      
            
            // Draw title
            framework().display.print_at(PADDING as i64, (y + PADDING) as i64, &title.clone());

            // Draw expression
            framework().rbop_location_x = PADDING as i64;
            framework().rbop_location_y = (y + 30 + PADDING) as i64;
            framework().draw_all(
                &rbop_ctx.root, 
                Some(&mut rbop_ctx.nav_path.to_navigator()),
                rbop_ctx.viewport.as_ref(),
            );

            // Push to screen
            framework().display.draw();

            // Poll for input
            if let Some(input) = framework().buttons.wait_press() {
                if OSInput::Exe == input {
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
    ) -> Number {
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
                    os().ui_text_dialog(&s);
                }
            }
        }
    }

    /// Opens a text dialog in the centre of the screen which can be dismissed with EXE.
    pub fn ui_text_dialog(&mut self, s: &str) {
        const H_PADDING: i64 = 30;
        const H_INNER_PADDING: i64 = 10;
        const V_PADDING: i64 = 10;
        let w = framework().display.width as i64 - H_PADDING * 2;
        let (lines, ch, h) = framework().display.wrap_text(s, w - H_INNER_PADDING * 2);
        let y_start = (framework().display.height as i64 - h) / 2;

        framework().display.draw_rect(
            H_PADDING, y_start,
            w, h + V_PADDING * 2,
            Colour::GREY, ShapeFill::Filled, 10
        );
        framework().display.draw_rect(
            H_PADDING, y_start,
            w, h + V_PADDING * 2,
            Colour::WHITE, ShapeFill::Hollow, 10
        );
        
        for (i, line) in lines.iter().enumerate() {
            framework().display.print_at(
                H_PADDING + H_INNER_PADDING, y_start + V_PADDING + ch * i as i64,
                line
            );
        }

        // Push to screen
        framework().display.draw();

        // Poll for input
        loop {
            if let Some(input) = framework().buttons.wait_press() {
                if OSInput::Exe == input {
                    break;
                }
            }
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum OSInput {
    Exe,
    Shift,
    List,

    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Delete,

    Digit(u8),

    Point,
    Parentheses,

    Add,
    Subtract,
    Multiply,
    Fraction,
    Power,

    TextMultiTapNew(char),
    TextMultiTapCycle(char),
}
