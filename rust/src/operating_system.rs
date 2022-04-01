use alloc::{boxed::Box, format, string::String, vec::Vec, rc::Rc};
use rbop::{Number, node::unstructured::{UnstructuredNodeRoot, Upgradable}, render::{Area, Renderer, Viewport, LayoutComputationProperties}};
use core::{cmp::max, mem, slice, marker::PhantomData, cell::{RefCell, RefMut}, borrow::{Borrow, BorrowMut}};

use crate::{
    applications::{Application, ApplicationList, menu::MenuApplication},
    // filesystem::{CalculationHistory, ChunkTable, Filesystem, RawStorage, Settings, FatInterface},
    interface::{Colour, ShapeFill, ApplicationFramework, DisplayInterface},
    // multi_tap::MultiTapState,
    // rbop_impl::RbopContext,
    // c_allocator::{MEMORY_USAGE, EXTERNAL_MEMORY_USAGE, MAX_MEMORY_USAGE, MAX_EXTERNAL_MEMORY_USAGE}
};

pub struct OperatingSystem<F: ApplicationFramework> {
    pub framework: F,

    // TODO: I don't think the operating system can hold application lists any more, since that 
    // would lead to recursive references (unless we use an Rc) - so where *do* we put them?

    pub application_list: ApplicationList<F>,
    pub menu: Option<Rc<RefCell<MenuApplication<F>>>>,
    pub showing_menu: bool,

    pub active_application: Option<Rc<RefCell<dyn Application<Framework = F>>>>,
    pub active_application_index: Option<usize>,

    // pub filesystem: Filesystem<'a>,
    // pub last_title_millis: u32,

    // pub text_mode: bool,
    // pub multi_tap: MultiTapState,
}

impl<F: ApplicationFramework> OperatingSystem<F> {
    pub const TITLE_BAR_HEIGHT: i64 = 30;
    
    pub fn new(framework: F) -> Self {
        Self {
            framework,
            application_list: ApplicationList::new(),
            active_application: None,
            active_application_index: None,
            menu: None, // TODO: initialise later
            showing_menu: true,
        }
    }

    /// Replaces the currently-running application with a new instance of the application at `index`
    /// in `application_list`.
    pub fn launch_application(&mut self, index: usize) {
        self.showing_menu = false;

        // TODO: destroy now unused

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
    // pub fn application_to_tick(&mut self) -> Rc<RefCell<dyn Application<Framework = F>>> {
    //     if self.showing_menu {
    //         self.menu.clone().expect("menu not configured yet")
    //     } else {
    //         // TODO: use menu here if None
    //         self.active_application.clone().unwrap()
    //     }
    // }

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

    /// Enables USB mass storage mode. The calculator will appear as a mass storage device, and hang
    /// until it is either ejected or the user presses DEL.
    /// Temporary, can be removed when driver interacts directly with storage.
    pub fn save_usb_mass_storage(&mut self) {
        self.framework.display_mut().fill_screen(Colour::BLACK);
        self.ui_draw_title("USB Mass Storage");
        let width = self.framework.display().width();
        self.framework.display_mut().print_centred(0, 100, width as i64, "Saving...");
        self.framework.display_mut().draw();

        // TODO
        // unsafe {
        //     os().filesystem.fat.write_all(slice::from_raw_parts_mut(
        //         framework().usb_mass_storage.fat12_filesystem,
        //         framework().usb_mass_storage.block_size * framework().usb_mass_storage.block_num
        //     )).unwrap();
        // }
    }

    /// Draws a title bar to the top of the screen, with the text `s`.
    pub fn ui_draw_title(&mut self, s: &str) {
        // let now_millis = (framework().millis)();
        // let millis_elapsed = now_millis - self.last_title_millis;
        // self.last_title_millis = now_millis;

        let width = self.framework.display().width();
        self.framework.display_mut().draw_rect(
            0, 0, width as i64, Self::TITLE_BAR_HEIGHT,
            Colour::ORANGE, ShapeFill::Filled, 0
        );

        // Draw title, according to settings
        // let frame_time = format!("{} ms", millis_elapsed);
        // let heap_usage = format!(
        //     "({}+{})[{}+{}]kB",
        //     unsafe { MEMORY_USAGE / 1000 },
        //     unsafe { EXTERNAL_MEMORY_USAGE / 1000 },
        //     unsafe { MAX_MEMORY_USAGE / 1000 },
        //     unsafe { MAX_EXTERNAL_MEMORY_USAGE / 1000 },
        // );

        // let settings = &os().filesystem.settings.values;
        // let title_text = match (settings.show_frame_time, settings.show_heap_usage) {
        //     (true, true) => format!("{} | {}", heap_usage, frame_time),
        //     (true, false) => frame_time,
        //     (false, true) => heap_usage,
        //     (false, false) => s.into(),
        // };

        self.framework.display_mut().print_at(5, 7, s);

        // Draw charge indicator
        // let charge_status = (framework().charge_status)();
        // let charge_bitmap = if charge_status == -1 { "power_usb".into() } else { format!("battery_{}", charge_status) };
        // self.framework.display().draw_bitmap(200, 6, &charge_bitmap);

        // // Draw text indicator
        // if os().text_mode {
        //     self.framework.display().draw_rect(145, 4, 50, 24, Colour::WHITE, ShapeFill::Hollow, 5);
        //     if os().multi_tap.shift {
        //         self.framework.display().print_at(149, 6, "TEXT");
        //     } else {
        //         self.framework.display().print_at(153, 6, "text");
        //     }
        // }
    }

    /// Opens a menu with the items in the slice `items`. The user can navigate the menu with the
    /// up and down keys, and select an item with EXE.
    /// Returns Some(the index of the item selected).
    /// These menus are typically to be opened with the LIST key. If `can_close` is true, pressing
    /// LIST will return None.
    pub fn ui_open_menu(&mut self, items: &[String], can_close: bool) -> Option<usize> {
        todo!()
        // const ITEM_GAP: i64 = 30;
        // let mut selected_index = 0;

        // loop {
        //     // Draw background
        //     let mut y = (self.framework.display().height() as i64 - ITEM_GAP * items.len() as i64 - 10) as i64;
        //     self.framework.display().draw_rect(0, y, 240, 400, Colour::GREY, ShapeFill::Filled, 10);
        //     self.framework.display().draw_rect(0, y, 240, 400, Colour::WHITE, ShapeFill::Hollow, 10);

        //     // Draw items
        //     y += 10;
        //     for (i, item) in items.iter().enumerate() {
        //         if i == selected_index {
        //             self.framework.display().draw_rect(
        //                 5, y, self.framework.display().width() as i64 - 5 * 2, 25,
        //                 Colour::BLUE, ShapeFill::Filled, 7
        //             );
        //         }
        //         self.framework.display().print_at(10, y as i64 + 4, item);

        //         y += ITEM_GAP;
        //     }

        //     self.framework.display().draw();

        //     if let Some(btn) = framework().buttons.wait_press() {
        //         match btn {
        //             OSInput::MoveUp => {
        //                 if selected_index == 0 {
        //                     selected_index = items.len() - 1;
        //                 } else {
        //                     selected_index -= 1;
        //                 }
        //             }
        //             OSInput::MoveDown => {
        //                 selected_index += 1;
        //                 selected_index %= items.len();
        //             }
        //             OSInput::Exe => return Some(selected_index),
        //             OSInput::List if can_close => return None,
        //             _ => (),
        //         }
        //     }
        // }
    }

    /// Opens an rbop input box with the given `title` and optionally starts the node tree at the
    /// given `root`. When the user presses EXE, returns the current node tree.
    pub fn ui_input_expression(&mut self, title: impl Into<String>, root: Option<UnstructuredNodeRoot>) -> UnstructuredNodeRoot {
        todo!()
        // const PADDING: u64 = 10;
        
        // let mut rbop_ctx = RbopContext {
        //     viewport: Some(Viewport::new(Area::new(
        //         self.framework.display().width - PADDING * 2,
        //         self.framework.display().height - PADDING * 2,
        //     ))),
        //     ..RbopContext::new()
        // };

        // if let Some(unr) = root {
        //     rbop_ctx.root = unr;
        // }

        // let title = title.into();

        // // Don't let the box get any shorter than the maximum height it has achieved, or you'll get
        // // ghost boxes if the height reduces since we don't redraw the whole frame
        // let mut minimum_height = 0;
        
        // loop {
        //     // Calculate layout in advance so we know height
        //     let layout = framework().layout(
        //         &rbop_ctx.root,
        //         Some(&mut rbop_ctx.nav_path.to_navigator()),
        //         LayoutComputationProperties::default(),
        //     );
        //     let height = max(layout.area.height, minimum_height);

        //     if height > minimum_height {
        //         minimum_height = height;
        //     }

        //     // Draw background
        //     let y = self.framework.display().height
        //         - height
        //         - 30
        //         - PADDING * 2;
        //     self.framework.display().draw_rect(0, y as i64, 240, 400, Colour::GREY, ShapeFill::Filled, 10);
        //     self.framework.display().draw_rect(0, y as i64, 240, 400, Colour::WHITE, ShapeFill::Hollow, 10);      
            
        //     // Draw title
        //     self.framework.display().print_at(PADDING as i64, (y + PADDING) as i64, &title.clone());

        //     // Draw expression
        //     framework().rbop_location_x = PADDING as i64;
        //     framework().rbop_location_y = (y + 30 + PADDING) as i64;
        //     framework().draw_all(
        //         &rbop_ctx.root, 
        //         Some(&mut rbop_ctx.nav_path.to_navigator()),
        //         rbop_ctx.viewport.as_ref(),
        //     );

        //     // Push to screen
        //     self.framework.display().draw();

        //     // Poll for input
        //     if let Some(input) = framework().buttons.wait_press() {
        //         if OSInput::Exe == input {
        //             return rbop_ctx.root;
        //         } else {
        //             rbop_ctx.input(input);
        //         }
        //     }
        // }
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
        todo!()
        // let title = title.into();
        // let mut unr = root;
        // loop {
        //     redraw();
        //     unr = Some(os().ui_input_expression(title.clone(), unr));
        //     match unr
        //         .as_ref()
        //         .unwrap()
        //         .upgrade()
        //         .map_err(|e| format!("{:?}", e))
        //         .and_then(|sn| sn
        //             .evaluate()
        //             .map_err(|e| format!("{:?}", e))) {
                
        //         Ok(d) => {
        //             return d;
        //         }
        //         Err(s) => {
        //             redraw();
        //             os().ui_text_dialog(&s);
        //         }
        //     }
        // }
    }

    /// Opens a text dialog in the centre of the screen which can be dismissed with EXE.
    pub fn ui_text_dialog(&mut self, s: &str) {
        todo!()
        // const H_PADDING: i64 = 30;
        // const H_INNER_PADDING: i64 = 10;
        // const V_PADDING: i64 = 10;
        // let w = self.framework.display().width as i64 - H_PADDING * 2;
        // let (lines, ch, h) = self.framework.display().wrap_text(s, w - H_INNER_PADDING * 2);
        // let y_start = (self.framework.display().height as i64 - h) / 2;

        // self.framework.display().draw_rect(
        //     H_PADDING, y_start,
        //     w, h + V_PADDING * 2,
        //     Colour::GREY, ShapeFill::Filled, 10
        // );
        // self.framework.display().draw_rect(
        //     H_PADDING, y_start,
        //     w, h + V_PADDING * 2,
        //     Colour::WHITE, ShapeFill::Hollow, 10
        // );
        
        // for (i, line) in lines.iter().enumerate() {
        //     self.framework.display().print_at(
        //         H_PADDING + H_INNER_PADDING, y_start + V_PADDING + ch * i as i64,
        //         line
        //     );
        // }

        // // Push to screen
        // self.framework.display().draw();

        // // Poll for input
        // loop {
        //     if let Some(input) = framework().buttons.wait_press() {
        //         if OSInput::Exe == input {
        //             break;
        //         }
        //     }
        // }
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
    Clear,

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

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct UIMenuItem {
    pub title: String,
    pub icon: String,
    pub toggle: Option<bool>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct UIMenu {
    pub items: Vec<UIMenuItem>,
    pub selected_index: usize,
    page_scroll_offset: usize,
}

impl UIMenu {
    const ITEMS_PER_PAGE: usize = 5;

    pub fn new(items: Vec<UIMenuItem>) -> Self {
        Self {
            items,
            selected_index: 0,
            page_scroll_offset: 0,
        }
    }

    pub fn draw(&self) {
        todo!()
        // // Draw items
        // let mut y = OperatingSystemInterface::TITLE_BAR_HEIGHT + 10;

        // // Bail early if no items
        // if self.items.is_empty() {
        //     self.framework.display().print_at(75, y, "No items");
        //     return;
        // }

        // for (i, item) in self.items.iter().enumerate().skip(self.page_scroll_offset).take(Self::ITEMS_PER_PAGE) {
        //     // Work out whether we need to wrap
        //     // TODO: not an exact width
        //     let (lines, _, _) = self.framework.display().wrap_text(&item.title, 120);

        //     if i == self.selected_index {
        //         self.framework.display().draw_rect(
        //             5, y, self.framework.display().width as i64 - 5 * 2 - 8, 54,
        //             Colour::BLUE, ShapeFill::Filled, 7
        //         );
        //     }

        //     if lines.len() == 1 {
        //         self.framework.display().print_at(65, y + 18, &lines[0]);
        //     } else {
        //         self.framework.display().print_at(65, y + 7, &lines[0]);
        //         self.framework.display().print_at(65, y + 28 , &lines[1]);
        //     }
        //     self.framework.display().draw_bitmap(7, y + 2, &item.icon);

        //     // Draw toggle, if necessary
        //     if let Some(toggle_position) = item.toggle {
        //         let toggle_bitmap_name = if toggle_position { "toggle_on" } else { "toggle_off" };
        //         self.framework.display().draw_bitmap(195, y + 20, toggle_bitmap_name);
        //     }

        //     y += 54;
        // }

        // // Draw scroll amount indicator
        // let scroll_indicator_column_height = 54 * Self::ITEMS_PER_PAGE;
        // let scroll_indicator_bar_height_per_item = scroll_indicator_column_height / self.items.len();
        // let scroll_indicator_bar_offset = scroll_indicator_bar_height_per_item * self.page_scroll_offset;
        // let scroll_indicator_bar_height = scroll_indicator_bar_height_per_item * Self::ITEMS_PER_PAGE;

        // self.framework.display().draw_rect(
        //     self.framework.display().width as i64 - 8, 40 + scroll_indicator_bar_offset as i64,
        //     4, scroll_indicator_bar_height as i64, Colour::DARK_BLUE, ShapeFill::Filled, 2
        // );        
    }

    pub fn move_up(&mut self) {
        if self.selected_index == 0 {
            // Wrap
            self.selected_index = self.items.len() - 1;

            if self.items.len() > Self::ITEMS_PER_PAGE {
                self.page_scroll_offset = self.items.len() - Self::ITEMS_PER_PAGE;
            } else {
                self.page_scroll_offset = 0;
            }
        } else {
            self.selected_index -= 1;

            // If scrolled off the screen, scroll up
            if self.selected_index < self.page_scroll_offset {
                self.page_scroll_offset -= 1;
            }
        }
    }

    pub fn move_down(&mut self) {
        self.selected_index += 1;

        // Wrap
        if self.selected_index == self.items.len() {
            self.selected_index = 0;
            self.page_scroll_offset = 0;
        }

        // If scrolled off the screen, scroll down
        if self.selected_index >= self.page_scroll_offset + Self::ITEMS_PER_PAGE {
            self.page_scroll_offset += 1;
        }
    }
}
