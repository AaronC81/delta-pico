use core::alloc::{GlobalAlloc, Layout};

use alloc::{vec, format};

use crate::{interface::{Colour, ShapeFill, ApplicationFramework, ButtonInput, DisplayInterface}, operating_system::{OSInput, UIMenu, UIMenuItem, os_accessor, OperatingSystem}, timer::Timer};
use super::{Application, ApplicationInfo};

// TODO: mostly unimplemented

pub struct SettingsApplication<F: ApplicationFramework + 'static> {
    os: *mut OperatingSystem<F>,
    menu: UIMenu<F>,
}

os_accessor!(SettingsApplication<F>);

impl<F: ApplicationFramework> Application for SettingsApplication<F> {
    type Framework = F;

    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "Settings".into(),
            visible: false,
        }
    }

    fn new(os: *mut OperatingSystem<F>) -> Self {
        Self {
            os,

            menu: UIMenu::new(os, vec![
                // UIMenuItem {
                //     title: "Show frame time".into(),
                //     icon: "settings_show_frame_time".into(),
                //     toggle: Some(os().filesystem.settings.values.show_frame_time),
                // },
                // UIMenuItem {
                //     title: "Show heap usage".into(),
                //     icon: "settings_show_memory_usage".into(),
                //     toggle: Some(os().filesystem.settings.values.show_heap_usage),
                // },
                // UIMenuItem {
                //     title: "Fire button press only".into(),
                //     icon: "settings_fire_button_press_only".into(),
                //     toggle: Some(os().filesystem.settings.values.fire_button_press_only),
                // },
                UIMenuItem {
                    title: "Graphics benchmark".into(),
                    icon: "settings_graphics_benchmark".into(),
                    toggle: None,
                },
                // UIMenuItem {
                //     title: "Run test suite".into(),
                //     icon: "settings_test".into(),
                //     toggle: None,
                // },
                // UIMenuItem {
                //     title: "Memory leak test".into(),
                //     icon: "settings_memory_leak".into(),
                //     toggle: None,
                // }
            ]),
        }
    }

    fn tick(&mut self) {
        self.os_mut().framework.display_mut().fill_screen(Colour::BLACK);
        self.os_mut().ui_draw_title("Settings");

        self.menu.draw();
        self.os_mut().framework.display_mut().draw();

        if let Some(btn) = self.os_mut().input() {
            match btn {
                OSInput::Button(ButtonInput::MoveUp) => self.menu.move_up(),
                OSInput::Button(ButtonInput::MoveDown) => self.menu.move_down(),
                OSInput::Button(ButtonInput::Exe) => self.change_selected_setting(),
                _ => (),
            }
        }
    }
}

impl<F: ApplicationFramework> SettingsApplication<F> {
    fn change_selected_setting(&mut self) {
        match self.menu.selected_index {
            0 => self.graphics_benchmark(),

            // 0 => self.toggle_setting(0, &mut os().filesystem.settings.values.show_frame_time),
            // 1 => self.toggle_setting(1, &mut os().filesystem.settings.values.show_heap_usage),
            // 2 => {
            //     // Show a warning if we're turning it on
            //     if !os().filesystem.settings.values.fire_button_press_only {
            //         os().ui_text_dialog("This setting is experimental! Responsiveness will improve, but frame times will become inaccurate, and some apps may break.");
            //     }
                
            //     self.toggle_setting(2, &mut os().filesystem.settings.values.fire_button_press_only)
            // },
            // 3 => self.graphics_benchmark(),
            // 4 => self.run_test_suite(),
            // 5 => self.leak_memory_until_panic(),
            
            _ => unreachable!()
        }
    }

    // fn toggle_setting(&mut self, index: usize, setting: &mut bool) {
    //     *setting = !*setting;
    //     self.menu.items[index].toggle = Some(*setting);
        
    //     os().filesystem.settings.save();
    // }

    fn graphics_benchmark(&self) {
        // TODO: We could test sprites too

        let mut fill_timer = Timer::new(self.os, "Fill");
        let mut rectangles_timer = Timer::new(self.os, "Rectangles");
        let mut text_timer = Timer::new(self.os, "Text");
        let mut draw_timer = Timer::new(self.os, "Draw");

        // Run a simple drawing test many times
        for _ in 0..50 {
            // Clear the screen
            fill_timer.start();
            self.os_mut().framework.display_mut().fill_screen(Colour::BLACK);
            fill_timer.stop();

            // Draw some rectangles
            rectangles_timer.start();
            self.os_mut().framework.display_mut().draw_rect(
                20, 20, 60, 60, Colour::ORANGE,
                ShapeFill::Filled, 0
            );
            self.os_mut().framework.display_mut().draw_rect(
                80, 20, 60, 60, Colour::BLUE,
                ShapeFill::Filled, 11
            );
            self.os_mut().framework.display_mut().draw_rect(
                20, 80, 60, 60, Colour::WHITE,
                ShapeFill::Hollow, 0
            );
            self.os_mut().framework.display_mut().draw_rect(
                80, 80, 60, 60, Colour::RED,
                ShapeFill::Hollow, 11
            );
            rectangles_timer.stop();

            // Draw some text
            text_timer.start();
            self.os_mut().framework.display_mut().print_at(30, 50, "Hello, world!\nHello again.");
            self.os_mut().framework.display_mut().print_at(30, 110, "Another line...\nOne final line.");
            text_timer.stop();

            // Draw to screen
            draw_timer.start();
            self.os_mut().framework.display_mut().draw();
            draw_timer.stop();
        }

        let total =
            fill_timer.elapsed
            + rectangles_timer.elapsed
            + text_timer.elapsed
            + draw_timer.elapsed;

        // Present the results
        self.os_mut().framework.display_mut().fill_screen(Colour::BLACK);
        self.os_mut().ui_draw_title("Results");

        self.os_mut().framework.display_mut().print_at(0, 40, &format!(
            "Total: {}\n\n{}{}{}{}\n(Lower is faster)",
            total, fill_timer, rectangles_timer, text_timer, draw_timer
        ));

        self.os_mut().framework.display_mut().print_centred(
            0, 290, self.os().framework.display().width(), "[EXE]: Close"
        );
        self.os_mut().framework.display_mut().draw();

        // Wait until EXE press
        loop {
            if let Some(OSInput::Button(ButtonInput::Exe)) = self.os_mut().input() {
                break;
            }
        }
    }

    // fn run_test_suite(&self) {
    //     tests::run_all_tests();
    //     os().ui_text_dialog("Tests passed!");
    // }

    // fn leak_memory_until_panic(&self) -> ! {
    //     todo!(); // TODO
    // }
}
