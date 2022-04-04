use alloc::{boxed::Box, format, string::{String, ToString}, vec::Vec, rc::Rc};
use az::SaturatingAs;
use rbop::{Number, node::unstructured::{UnstructuredNodeRoot, Upgradable}, render::{Area, Renderer, Viewport, LayoutComputationProperties}};
use core::{cmp::max, mem, slice, marker::PhantomData, cell::{RefCell, RefMut}, borrow::{Borrow, BorrowMut}};

use crate::{
    applications::{Application, ApplicationList, menu::MenuApplication},
    // filesystem::{CalculationHistory, ChunkTable, Filesystem, RawStorage, Settings, FatInterface},
    interface::{Colour, ShapeFill, ApplicationFramework, DisplayInterface, ButtonInput, ButtonsInterface, ButtonEvent}, multi_tap::MultiTapState, filesystem::{Filesystem, Settings, RawStorage, SettingsValues}, graphics::Sprite, rbop_impl::RbopContext,
    // multi_tap::MultiTapState,
    // rbop_impl::RbopContext,
    // c_allocator::{MEMORY_USAGE, EXTERNAL_MEMORY_USAGE, MAX_MEMORY_USAGE, MAX_EXTERNAL_MEMORY_USAGE}
};

pub struct OperatingSystem<F: ApplicationFramework + 'static> {
    pub framework: F,

    // TODO: I don't think the operating system can hold application lists any more, since that 
    // would lead to recursive references (unless we use an Rc) - so where *do* we put them?

    pub application_list: ApplicationList<F>,
    pub menu: Option<MenuApplication<F>>,
    pub showing_menu: bool,

    pub active_application: Option<Box<dyn Application<Framework = F>>>,
    pub active_application_index: Option<usize>,

    pub filesystem: Filesystem<F>,
    // pub last_title_millis: u32,

    pub text_mode: bool,
    pub multi_tap: MultiTapState,

    pub display_sprite: Sprite,
}

impl<F: ApplicationFramework> OperatingSystem<F> {
    pub const TITLE_BAR_HEIGHT: u16 = 30;
    
    pub fn new(mut framework: F) -> Self {
        let display_width = framework.display().width();
        let display_height = framework.display().height();

        Self {
            framework,

            application_list: ApplicationList::new(),
            active_application: None,
            active_application_index: None,

            menu: None, // TODO: initialise later
            showing_menu: true,

            filesystem: Filesystem {
                settings: Settings::new(
                    RawStorage {
                        os: core::ptr::null_mut(),
                        start_address: 0,
                        length: Settings::<F>::MINIMUM_STORAGE_SIZE,
                    }
                )
                // settings: Settings {
                //     storage: RawStorage {
                //         os: core::ptr::null_mut(),
                //         start_address: 0,
                //         length: Settings::<F>::MINIMUM_STORAGE_SIZE,
                //     },
                //     values: SettingsValues::default(),
                // }
            },

            text_mode: false,
            multi_tap: MultiTapState::new(),

            display_sprite: Sprite::new(display_width, display_height),
        }
    }

    /// Performs second-stage initialisation tasks which require a pointer to this OS instance.
    /// This *MUST* be called shortly after `new`, or nasty UB and null dereferences will occur.
    pub fn second_init(&mut self) {
        // Set up cyclic raw pointers
        let ptr = self as *mut _;
        self.application_list.os = ptr;
        self.filesystem.settings.storage.os = ptr;

        // Load storage values
        self.filesystem.settings.load_into_self();
    }

    /// Replaces the currently-running application with a new instance of the application at `index`
    /// in `application_list`.
    pub fn launch_application(&mut self, index: usize) {
        self.showing_menu = false;

        // TODO: destroy now unused

        self.active_application_index = Some(index);
        self.active_application = Some(self.application_list.applications[index].1(self.application_list.os));
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
    pub fn application_to_tick(&mut self) -> &mut dyn Application<Framework = F> {
        if self.showing_menu {
            self.menu.as_mut().unwrap()
        } else {
            self.active_application.as_mut()
                .map(|x| x.as_mut())
                .unwrap_or(self.menu.as_mut().unwrap())
        }
    }

    /// Draws the display sprite, updating the (physical) display after a set of drawing operations.
    pub fn draw(&mut self) {
        self.framework.display_mut().draw_display_sprite(&self.display_sprite)
    }

    /// Toggles whether the global menu is currently being shown.
    pub fn toggle_menu(&mut self) {
        self.showing_menu = !self.showing_menu;
    }

    /// Enables USB mass storage mode. The calculator will appear as a mass storage device, and hang
    /// until it is either ejected or the user presses DEL.
    /// Temporary, can be removed when driver interacts directly with storage.
    pub fn save_usb_mass_storage(&mut self) {
        self.display_sprite.fill(Colour::BLACK);
        self.ui_draw_title("USB Mass Storage");
        let width = self.framework.display().width();
        self.display_sprite.print_centred(0, 100, width, "Saving...");
        self.draw();

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
        self.display_sprite.draw_rect(
            0, 0, self.display_sprite.width, Self::TITLE_BAR_HEIGHT,
            Colour::ORANGE, ShapeFill::Filled, 0
        );

        // TODO: frame time
        let s = if self.filesystem.settings.values.show_heap_usage {
            let (used, total) = self.framework.memory_usage();
            format!("{}/{}kB", used / 1000, total / 1000)
        } else {
            s.to_string()
        };

        self.display_sprite.print_at(5, 7, &s);

        // Draw charge indicator
        // let charge_status = (framework().charge_status)();
        // let charge_bitmap = if charge_status == -1 { "power_usb".into() } else { format!("battery_{}", charge_status) };
        // self.framework.display().draw_bitmap(200, 6, &charge_bitmap);

        // Draw text indicator
        if self.text_mode {
            self.display_sprite.draw_rect(145, 4, 50, 24, Colour::WHITE, ShapeFill::Hollow, 5);
            if self.multi_tap.shift {
                self.display_sprite.print_at(149, 6, "TEXT");
            } else {
                self.display_sprite.print_at(153, 6, "text");
            }
        }
    }

    /// Opens a menu with the items in the slice `items`. The user can navigate the menu with the
    /// up and down keys, and select an item with EXE.
    /// Returns Some(the index of the item selected).
    /// These menus are typically to be opened with the LIST key. If `can_close` is true, pressing
    /// LIST will return None.
    pub fn ui_open_menu(&mut self, items: &[String], can_close: bool) -> Option<usize> {
        const ITEM_GAP: i16 = 30;
        let mut selected_index = 0;

        loop {
            // Draw background
            let mut y = self.framework.display().height() as i16 - ITEM_GAP * items.len() as i16 - 10;
            self.display_sprite.draw_rect(0, y, 240, 400, Colour::GREY, ShapeFill::Filled, 10);
            self.display_sprite.draw_rect(0, y, 240, 400, Colour::WHITE, ShapeFill::Hollow, 10);

            // Draw items
            y += 10;
            for (i, item) in items.iter().enumerate() {
                if i == selected_index {
                    let width = self.framework.display().width();
                    self.display_sprite.draw_rect(
                        5, y as i16, width - 5 * 2, 25,
                        Colour::BLUE, ShapeFill::Filled, 7
                    );
                }
                self.display_sprite.print_at(10, y + 4, item);

                y += ITEM_GAP;
            }

            self.draw();

            if let Some(btn) = self.input() {
                match btn {
                    OSInput::Button(ButtonInput::MoveUp) => {
                        if selected_index == 0 {
                            selected_index = items.len() - 1;
                        } else {
                            selected_index -= 1;
                        }
                    }
                    OSInput::Button(ButtonInput::MoveDown) => {
                        selected_index += 1;
                        selected_index %= items.len();
                    }
                    OSInput::Button(ButtonInput::Exe) => return Some(selected_index),
                    OSInput::Button(ButtonInput::List) if can_close => return None,
                    _ => (),
                }
            }
        }
    }

    /// Opens an rbop input box with the given `title` and optionally starts the node tree at the
    /// given `root`. When the user presses EXE, returns the current node tree.
    pub fn ui_input_expression(&mut self, title: &str, root: Option<UnstructuredNodeRoot>) -> UnstructuredNodeRoot {
        const PADDING: i16 = 10;
        
        let mut expression_sprite = Sprite::empty();
        let mut rbop_ctx = RbopContext {
            viewport: Some(Viewport::new(Area::new(
                (self.display_sprite.width - PADDING as u16 * 2).into(),
                (self.display_sprite.height - PADDING as u16 * 2).into(),
            ))),
            ..RbopContext::new(self as *mut _, &mut expression_sprite)
        };

        if let Some(unr) = root {
            rbop_ctx.root = unr;
        }

        // Don't let the box get any shorter than the maximum height it has achieved, or you'll get
        // ghost boxes if the height reduces since we don't redraw the whole frame
        let mut minimum_height = 0u16;
        
        loop {
            // Calculate layout in advance so we know size
            let layout = rbop_ctx.sprite.layout(
                &rbop_ctx.root,
                Some(&mut rbop_ctx.nav_path.to_navigator()),
                LayoutComputationProperties::default(),
            );
            let height: u16 = max(layout.area.height, minimum_height.into()).saturating_as::<u16>();
            let width = layout.area.width.saturating_as::<u16>();

            if height > minimum_height {
                minimum_height = height;
            }

            // Resize the sprite now that we know the size
            rbop_ctx.sprite.resize(width, height);

            // Draw background of dialog
            let y = self.display_sprite.height
                - height
                - 30
                - PADDING as u16 * 2;
            self.display_sprite.draw_rect(0, y.saturating_as::<i16>(), 240, 400, Colour::GREY, ShapeFill::Filled, 10);
            self.display_sprite.draw_rect(0, y.saturating_as::<i16>(), 240, 400, Colour::WHITE, ShapeFill::Hollow, 10);      
            
            // Draw title
            self.display_sprite.print_at(PADDING, y.saturating_as::<i16>() + PADDING, &title);

            // Draw background and expression to sprite
            rbop_ctx.sprite.fill(Colour::GREY);
            rbop_ctx.sprite.draw_all(
                &rbop_ctx.root, 
                Some(&mut rbop_ctx.nav_path.to_navigator()),
                rbop_ctx.viewport.as_ref(),
            );

            // Draw sprite to screen
            self.display_sprite.draw_sprite(PADDING, y as i16 + 30 + PADDING, &mut rbop_ctx.sprite);

            // Update screen
            self.draw();

            // Poll for input
            if let Some(input) = self.input() {
                if input == OSInput::Button(ButtonInput::Exe) {
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
        title: &str,
        root: Option<UnstructuredNodeRoot>,
        mut redraw: impl FnMut(),
    ) -> Number {
        let title = title.into();
        let mut unr = root;
        loop {
            redraw();
            unr = Some(self.ui_input_expression(title, unr));
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
                    self.ui_text_dialog(&s);
                }
            }
        }
    }

    /// Opens a text dialog in the centre of the screen which can be dismissed with EXE.
    pub fn ui_text_dialog(&mut self, s: &str) {
        const H_PADDING: u16 = 30;
        const H_INNER_PADDING: u16 = 10;
        const V_PADDING: u16 = 10;

        let w = self.display_sprite.width - H_PADDING * 2;
        let (lines, ch, h) = self.display_sprite.wrap_text(s, w - H_INNER_PADDING * 2);
        let h = h as u16;
        let y_start = (self.display_sprite.height - h) / 2;

        self.display_sprite.draw_rect(
            H_PADDING as i16, y_start as i16,
            w, h + V_PADDING as u16 * 2,
            Colour::GREY, ShapeFill::Filled, 10
        );
        self.display_sprite.draw_rect(
            H_PADDING as i16, y_start as i16,
            w, h + V_PADDING as u16 * 2,
            Colour::WHITE, ShapeFill::Hollow, 10
        );
        
        for (i, line) in lines.iter().enumerate() {
            self.display_sprite.print_at(
                (H_PADDING + H_INNER_PADDING) as i16,
                y_start as i16 + V_PADDING as i16 + ch as i16 * i as i16,
                line
            );
        }

        // Push to screen
        self.draw();

        // Poll for input
        loop {
            if let Some(input) = self.input() {
                if OSInput::Button(ButtonInput::Exe) == input {
                    break;
                }
            }
        }
    }

    /// Utility method to translate a `ButtonInput` to an `OSInput`.
    /// 
    /// This may have a variety of side effects, including opening/closing menus or changing
    /// multitap state. As such, it should be called only for a *press* and not a release.
    fn button_input_to_os_input(&mut self, input: ButtonInput) -> Option<OSInput> {
        let mut result = match input {
            // Special cases
            ButtonInput::Menu => {
                self.toggle_menu();
                return None
            }
            ButtonInput::Text => {
                self.text_mode = !self.text_mode;
                return None
            }
            ButtonInput::None => return None,

            btn => Some(OSInput::Button(btn)),
        };

        // Intercept if a digit was pressed in text mode - this needs to be converted to a
        // character according to the OS' multi-tap state
        if self.text_mode {
            if let Some(r@OSInput::Button(ButtonInput::Digit(_))) = result {
                result = self.multi_tap.input(r);
            } else {
                // Make sure we don't cycle the wrong character if we e.g. move with the arrows
                self.multi_tap.drop_keypress();
            }
        }

        return result
    }

    pub fn input(&mut self) -> Option<OSInput> {
        loop {
            let event = self.framework.buttons_mut().wait_event();
            if let ButtonEvent::Press(btn_input) = event {
                return self.button_input_to_os_input(btn_input)
            }
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum OSInput {
    Button(ButtonInput),
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
pub struct UIMenu<F: ApplicationFramework + 'static> {
    os: *mut OperatingSystem<F>,
    pub items: Vec<UIMenuItem>,
    pub selected_index: usize,
    page_scroll_offset: usize,
}

impl<F: ApplicationFramework> UIMenu<F> {
    const ITEMS_PER_PAGE: usize = 5;

    pub fn new(os: *mut OperatingSystem<F>, items: Vec<UIMenuItem>) -> Self {
        Self {
            os,
            items,
            selected_index: 0,
            page_scroll_offset: 0,
        }
    }

    pub fn draw(&mut self) {
        // Draw items
        let mut y = (OperatingSystem::<F>::TITLE_BAR_HEIGHT + 10) as i16;

        // Bail early if no items
        if self.items.is_empty() {
            self.os_mut().display_sprite.print_at(75, y, "No items");
            return;
        }

        for (i, item) in self.items.iter().enumerate().skip(self.page_scroll_offset).take(Self::ITEMS_PER_PAGE) {
            // Work out whether we need to wrap
            // TODO: not an exact width
            let (lines, _, _) = self.os_mut().display_sprite.wrap_text(&item.title, 120);

            if i == self.selected_index {
                self.os_mut().display_sprite.draw_rect(
                    5, y, self.os().framework.display().width() - 5 * 2 - 8, 54,
                    Colour::BLUE, ShapeFill::Filled, 7
                );
            }

            if lines.len() == 1 {
                self.os_mut().display_sprite.print_at(65, y + 18, &lines[0]);
            } else {
                self.os_mut().display_sprite.print_at(65, y + 7, &lines[0]);
                self.os_mut().display_sprite.print_at(65, y + 28 , &lines[1]);
            }
            self.os_mut().display_sprite.draw_bitmap(7, y + 2, &item.icon);

            // Draw toggle, if necessary
            if let Some(toggle_position) = item.toggle {
                let toggle_bitmap_name = if toggle_position { "toggle_on" } else { "toggle_off" };
                self.os_mut().display_sprite.draw_bitmap(195, y + 20, toggle_bitmap_name);
            }

            y += 54;
        }

        // Draw scroll amount indicator
        let scroll_indicator_column_height = 54 * Self::ITEMS_PER_PAGE;
        let scroll_indicator_bar_height_per_item = scroll_indicator_column_height / self.items.len();
        let scroll_indicator_bar_offset = scroll_indicator_bar_height_per_item * self.page_scroll_offset;
        let scroll_indicator_bar_height = scroll_indicator_bar_height_per_item * core::cmp::min(Self::ITEMS_PER_PAGE, self.items.len());

        self.os_mut().display_sprite.draw_rect(
            self.os_mut().display_sprite.width as i16 - 8, 40 + scroll_indicator_bar_offset as i16,
            4, scroll_indicator_bar_height as u16, Colour::DARK_BLUE, ShapeFill::Filled, 2
        );
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

    fn os(&self) -> &OperatingSystem<F> { unsafe { &*self.os } }
    fn os_mut(&self) -> &mut OperatingSystem<F> { unsafe { &mut *self.os } }
}

macro_rules! os_accessor {
    ($n:ty) => {
        impl<F: ApplicationFramework> $n {
            #[allow(unused)]
            fn os(&self) -> &OperatingSystem<F> { unsafe { &*self.os } }

            #[allow(unused)]
            fn os_mut(&self) -> &mut OperatingSystem<F> { unsafe { &mut *self.os } }        
        }
    };
}
pub(crate) use os_accessor;
