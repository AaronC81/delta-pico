use alloc::{vec, vec::Vec};

use crate::{interface::Colour, operating_system::{OSInput, UIMenu, UIMenuItem, os}};
use super::{Application, ApplicationInfo};
use crate::interface::framework;

pub struct MenuApplication {
    menu: UIMenu,
}

impl Application for MenuApplication {
    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "Menu".into(),
            visible: false,
        }
    }

    fn new() -> Self where Self: Sized {
        Self {
            menu: UIMenu::new(vec![]),
        }
    }

    fn tick(&mut self) {
        framework().display.fill_screen(Colour::BLACK);
        os().ui_draw_title("Menu");

        // Doesn't work to assign during `new` for some reason, so do this instead
        self.menu.items = os().application_list.applications
            .iter()
            .map(|(app, _)| UIMenuItem {
                title: app.name.clone(),
                icon: app.icon_name(),
                toggle: None,
            })
            .collect::<Vec<_>>();
        self.menu.draw();
        framework().display.draw();

        if let Some(btn) = framework().buttons.wait_press() {
            match btn {
                OSInput::MoveUp => self.menu.move_up(),
                OSInput::MoveDown => self.menu.move_down(),
                OSInput::Exe => os().launch_application(self.menu.selected_index),
                _ => (),
            }
        }
    }
}
