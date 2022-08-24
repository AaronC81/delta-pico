use alloc::{vec, format};
use rbop::node::structured::AngleUnit;

use crate::{interface::{Colour, ShapeFill, ApplicationFramework, ButtonInput, DisplayInterface}, operating_system::{OSInput, FullPageMenu, FullPageMenuItem, os_accessor, OperatingSystem, OperatingSystemPointer, FullPageMenuItemDecorator}, timer::Timer};
use super::{Application, ApplicationInfo};

// TODO: mostly unimplemented

pub struct SettingsApplication<F: ApplicationFramework + 'static> {
    os: OperatingSystemPointer<F>,
    menu: FullPageMenu<F>,
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

    fn new(os: OperatingSystemPointer<F>) -> Self {
        let mut result = Self {
            os,
            menu: FullPageMenu::new(os, vec![]),
        };
        result.build_menu();
        result
    }

    fn tick(&mut self) {
        self.os_mut().display_sprite.fill(Colour::BLACK);
        self.os_mut().ui_draw_title("Settings");

        self.menu.draw();
        self.os_mut().draw();

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
    fn build_menu(&mut self) {
        self.menu.items = vec![
            FullPageMenuItem {
                title: format!("Angle unit: {}", self.os().filesystem.settings.values.angle_unit),
                icon: "settings_angle_unit".into(),
                decorator: FullPageMenuItemDecorator::None,
            },
            FullPageMenuItem {
                title: "Show frame time".into(),
                icon: "settings_show_frame_time".into(),
                decorator: FullPageMenuItemDecorator::Toggle(self.os().filesystem.settings.values.show_frame_time),
            },
            FullPageMenuItem {
                title: "Show heap usage".into(),
                icon: "settings_show_memory_usage".into(),
                decorator: FullPageMenuItemDecorator::Toggle(self.os().filesystem.settings.values.show_heap_usage),
            },
            FullPageMenuItem {
                title: "Graphics benchmark".into(),
                icon: "settings_graphics_benchmark".into(),
                decorator: FullPageMenuItemDecorator::None,
            },
        ];
    }

    fn change_selected_setting(&mut self) {
        let setting_value: &mut bool;
        let index: usize;

        match self.menu.selected_index {
            0 => {
                let new_setting_value = match self.os().filesystem.settings.values.angle_unit {
                    AngleUnit::Degree => AngleUnit::Radian,
                    AngleUnit::Radian => AngleUnit::Degree,
                };
                self.os_mut().filesystem.settings.values.angle_unit = new_setting_value;
                self.os_mut().filesystem.settings.save();

                self.menu.items[0].title = format!("Angle unit: {}", new_setting_value);
                return;
            }
            1 => {
                setting_value = &mut self.os_mut().filesystem.settings.values.show_frame_time;
                index = 1;
            }
            2 => {
                setting_value = &mut self.os_mut().filesystem.settings.values.show_heap_usage;
                index = 2;
            }
            3 => {
                self.graphics_benchmark();
                return
            }
            
            _ => unreachable!()
        }

        *setting_value = !*setting_value;
        self.menu.items[index].decorator = FullPageMenuItemDecorator::Toggle(*setting_value);

        self.os_mut().filesystem.settings.save();
    }

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
            self.os_mut().display_sprite.fill(Colour::BLACK);
            fill_timer.stop();

            // Draw some rectangles
            rectangles_timer.start();
            self.os_mut().display_sprite.draw_rect(
                20, 20, 60, 60, Colour::ORANGE,
                ShapeFill::Filled, 0
            );
            self.os_mut().display_sprite.draw_rect(
                80, 20, 60, 60, Colour::BLUE,
                ShapeFill::Filled, 11
            );
            self.os_mut().display_sprite.draw_rect(
                20, 80, 60, 60, Colour::WHITE,
                ShapeFill::Hollow, 0
            );
            self.os_mut().display_sprite.draw_rect(
                80, 80, 60, 60, Colour::RED,
                ShapeFill::Hollow, 11
            );
            rectangles_timer.stop();

            // Draw some text
            text_timer.start();
            self.os_mut().display_sprite.print_at(30, 50, "Hello, world!\nHello again.");
            self.os_mut().display_sprite.print_at(30, 110, "Another line...\nOne final line.");
            text_timer.stop();

            // Draw to screen
            draw_timer.start();
            self.os_mut().draw();
            draw_timer.stop();
        }

        let total =
            fill_timer.elapsed
            + rectangles_timer.elapsed
            + text_timer.elapsed
            + draw_timer.elapsed;

        // Present the results
        self.os_mut().display_sprite.fill(Colour::BLACK);
        self.os_mut().ui_draw_title("Results");

        self.os_mut().display_sprite.print_at(0, 40, &format!(
            "Total: {}\n\n{}{}{}{}\n(Lower is faster)",
            total, fill_timer, rectangles_timer, text_timer, draw_timer
        ));

        self.os_mut().display_sprite.print_centred(
            0, 290, self.os().framework.display().width(), "[EXE]: Close"
        );
        self.os_mut().draw();

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
