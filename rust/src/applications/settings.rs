use alloc::vec;

use crate::{interface::Colour, operating_system::{OSInput, UIMenu, UIMenuItem, os}};
use super::{Application, ApplicationInfo};
use crate::interface::framework;

pub struct SettingsApplication {
    menu: UIMenu,
}

impl Application for SettingsApplication {
    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "Settings".into(),
            visible: false,
        }
    }

    fn new() -> Self where Self: Sized {
        Self {
            menu: UIMenu::new(vec![
                UIMenuItem {
                    title: "Show frame time".into(),
                    icon: "settings_show_frame_time".into(),
                    toggle: Some(os().filesystem.settings.values.show_frame_time),
                },
                UIMenuItem {
                    title: "Show heap usage".into(),
                    icon: "settings_show_memory_usage".into(),
                    toggle: Some(os().filesystem.settings.values.show_heap_usage),
                },
                UIMenuItem {
                    title: "Fire button press only".into(),
                    icon: "settings_fire_button_press_only".into(),
                    toggle: Some(os().filesystem.settings.values.fire_button_press_only),
                }
            ]),
        }
    }

    fn tick(&mut self) {
        framework().display.fill_screen(Colour::BLACK);
        os().ui_draw_title("Settings");

        self.menu.draw();
        framework().display.draw();

        if let Some(btn) = framework().buttons.wait_press() {
            match btn {
                OSInput::MoveUp => self.menu.move_up(),
                OSInput::MoveDown => self.menu.move_down(),
                OSInput::Exe => self.change_selected_setting(),
                _ => (),
            }
        }
    }
}

impl SettingsApplication {
    fn change_selected_setting(&mut self) {
        match self.menu.selected_index {
            0 => self.toggle_setting(0, &mut os().filesystem.settings.values.show_frame_time),
            1 => self.toggle_setting(1, &mut os().filesystem.settings.values.show_heap_usage),
            2 => {
                // Show a warning if we're turning it on
                if !os().filesystem.settings.values.fire_button_press_only {
                    os().ui_text_dialog("This setting is experimental! Responsiveness will improve, but frame times will become inaccurate, and some apps may break.");
                }
                
                self.toggle_setting(2, &mut os().filesystem.settings.values.fire_button_press_only)
            },
            _ => unreachable!()
        }
    }

    fn toggle_setting(&mut self, index: usize, setting: &mut bool) {
        *setting = !*setting;
        self.menu.items[index].toggle = Some(*setting);
        
        os().filesystem.settings.save();
    }
}
