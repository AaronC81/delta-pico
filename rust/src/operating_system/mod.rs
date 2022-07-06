use core::ops::{DerefMut};

use alloc::{boxed::Box, format, vec::Vec};

use crate::{applications::{Application, ApplicationList, menu::MenuApplication, }, interface::{Colour, ShapeFill, ApplicationFramework, DisplayInterface}, multi_tap::MultiTapState, filesystem::{Filesystem, Settings, RawStorage, CHUNK_SIZE, CHUNK_ADDRESS_SIZE, ChunkTable, CalculationHistory}, graphics::Sprite};

mod pointer;
pub use pointer::*;

mod full_page_menu;
pub use full_page_menu::*;

mod input;
pub use input::*;

mod ui;
pub use ui::*;

pub struct OperatingSystem<F: ApplicationFramework + 'static> {
    pub ptr: OperatingSystemPointer<F>,
    pub framework: F,
    
    pub application_list: ApplicationList<F>,
    pub menu: Option<MenuApplication<F>>,
    pub showing_menu: bool,

    pub active_application: Option<Box<dyn Application<Framework = F>>>,
    pub active_application_index: Option<usize>,

    pub filesystem: Filesystem<F>,

    pub input_shift: bool,
    pub text_mode: bool,
    pub multi_tap: MultiTapState<F>,
    pub virtual_input_queue: Vec<Option<OSInput>>,

    pub display_sprite: Sprite,
    pub last_input_millis: u64,
}

impl<F: ApplicationFramework> OperatingSystem<F> {
    pub const TITLE_BAR_HEIGHT: u16 = 30;
    
    pub fn new(framework: F) -> Self {
        let display_width = framework.display().width();
        let display_height = framework.display().height();

        Self {
            ptr: OperatingSystemPointer::none(),
            framework,

            application_list: ApplicationList::new(),
            active_application: None,
            active_application_index: None,

            menu: None, // TODO: initialise later
            showing_menu: true,

            filesystem: Filesystem {
                settings: Settings::new(
                    RawStorage {
                        os: OperatingSystemPointer::none(),
                        start_address: 0,
                        length: Settings::<F>::MINIMUM_STORAGE_SIZE,
                    }
                ),
                
                calculations: CalculationHistory {
                    table: ChunkTable {
                        start_address: 0x1000,
                        chunks: 1024,
                        storage: RawStorage {
                            os: OperatingSystemPointer::none(),
                            start_address: 0x1000,

                            length:
                                CHUNK_SIZE * 1024
                                + 1024 / 8
                                + CHUNK_ADDRESS_SIZE * 1024,
                        },
                    }
                },
            },

            text_mode: false,
            multi_tap: MultiTapState::new(OperatingSystemPointer::none()),
            input_shift: false,
            virtual_input_queue: Vec::new(),

            display_sprite: Sprite::new(display_width, display_height),
            last_input_millis: 0,
        }
    }

    /// Performs second-stage initialisation tasks which require a pointer to this OS instance.
    /// This *MUST* be called shortly after `new`, or nasty UB and null dereferences will occur.
    pub fn second_init(mut ptr: OperatingSystemPointer<F>) {
        // Set up cyclic raw pointers
        ptr.deref_mut().ptr = ptr;
        ptr.application_list.os = ptr;
        ptr.filesystem.settings.storage.os = ptr;
        ptr.filesystem.calculations.table.storage.os = ptr;
        ptr.multi_tap.os = ptr;

        // Load storage values
        ptr.filesystem.settings.load_into_self();
    }

    /// Replaces the currently-running application with a new instance of the application at `index`
    /// in `application_list`.
    /// 
    /// NOTE: If called from an application, then **this will invalidate `self` once it returns**.
    /// The operating system owns the active application (through a trait object), so when a new
    /// application is launched, the current one will be dropped. The borrow checker would normally
    /// prevent a situation like this, but applications hold their OS reference through a raw
    /// pointer, so it can't. Be careful!
    pub fn launch_application(&mut self, index: usize) {
        self.showing_menu = false;

        self.active_application_index = Some(index);
        self.active_application = Some(self.application_list.applications[index].1(self.application_list.os));
    }

    /// Launches an application by its name.
    /// 
    /// The same **important safety warning** as `launch_application` applies!
    pub fn launch_application_by_name(&mut self, name: &str) {
        self.launch_application(
            self.application_list.applications
                .iter()
                .enumerate()
                .find(|(_, (app, _))| app.name == name)
                .unwrap()
                .0
        );
    }
    
    /// Restarts the current application. If none is open, panics.
    #[must_use =
        "`restart_application` will drop the current application, making it unsafe to continue \
        executing code within its methods, so the calling function should `return` here"
    ]
    pub fn restart_application(&mut self) {
        if let Some(index) = self.active_application_index {
            self.launch_application(index);
        } else {
            panic!("no application running to restart");
        }
    }

    /// Returns a reference to the application which should be ticked. This is typically the running
    /// application, unless showing the menu, in which case it is the menu application itself.
    #[allow(clippy::or_fun_call)] // Suggestion causes borrow checker issues
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
        const DEBUG_PANEL_WIDTH: i16 = 80;

        // Draw frame time, if enabled
        if self.filesystem.settings.values.show_frame_time {
            let now_millis = self.framework.millis();
            let millis_elapsed = now_millis - self.last_input_millis;
            self.display_sprite.draw_rect(
                self.display_sprite.width as i16 - DEBUG_PANEL_WIDTH, 0,
                DEBUG_PANEL_WIDTH as u16, Self::TITLE_BAR_HEIGHT / 2,
                Colour::BLACK, ShapeFill::Filled, 0,
            );
            self.display_sprite.with_font(&crate::font_data::DroidSans14, |sprite| {
                sprite.print_at(
                    sprite.width as i16 - DEBUG_PANEL_WIDTH, 0,
                    &format!("{}ms", millis_elapsed)
                );
            });
        }

        // Draw memory usage, if enabled
        if self.filesystem.settings.values.show_heap_usage {
            let (used, total) = self.framework.memory_usage();
            self.display_sprite.draw_rect(
                self.display_sprite.width as i16 - DEBUG_PANEL_WIDTH, Self::TITLE_BAR_HEIGHT as i16 / 2,
                DEBUG_PANEL_WIDTH as u16, Self::TITLE_BAR_HEIGHT / 2,
                Colour::BLACK, ShapeFill::Filled, 0,
            );
            self.display_sprite.with_font(&crate::font_data::DroidSans14, |sprite| {
                sprite.print_at(
                    sprite.width as i16 - DEBUG_PANEL_WIDTH, Self::TITLE_BAR_HEIGHT as i16 / 2,
                    &format!("{}/{}kB", used / 1000, total / 1000),
                );
            });
        }

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
}
