use alloc::{vec, vec::Vec};

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
                    toggle: Some(false),
                },
                UIMenuItem {
                    title: "Show heap usage".into(),
                    icon: "settings_show_memory_usage".into(),
                    toggle: Some(true),
                },
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
                OSInput::Exe => todo!(),
                _ => (),
            }
        }
    }
}
