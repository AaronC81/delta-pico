use alloc::{vec, vec::Vec};

use crate::{interface::{Colour, ApplicationFramework, DisplayInterface}, operating_system::{OSInput, UIMenu, UIMenuItem, OperatingSystem}};
use super::{Application, ApplicationInfo};

pub struct MenuApplication<'a, F: ApplicationFramework> {
    os: &'a mut OperatingSystem<'a, F>,
    menu: UIMenu,
}

impl<'a, F: ApplicationFramework> Application for MenuApplication<'a, F> {
    type Framework = F;

    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "Menu".into(),
            visible: false,
        }
    }

    fn new(os: &mut OperatingSystem<F>) -> Self where Self: Sized {
        Self {
            os,
            menu: UIMenu::new(vec![]),
        }
    }

    fn tick(&mut self) {
        self.os.framework.display().fill_screen(Colour::BLACK);
        self.os.ui_draw_title("Menu");

        // Doesn't work to assign during `new` for some reason, so do this instead
        self.menu.items = self.os.application_list.applications
            .iter()
            .map(|(app, _)| UIMenuItem {
                title: app.name.clone(),
                icon: app.icon_name(),
                toggle: None,
            })
            .collect::<Vec<_>>();
        self.menu.draw();
        self.os.framework.display().draw();

        // if let Some(btn) = framework().buttons.wait_press() {
        //     match btn {
        //         OSInput::MoveUp => self.menu.move_up(),
        //         OSInput::MoveDown => self.menu.move_down(),
        //         OSInput::Exe => os().launch_application(self.menu.selected_index),
        //         _ => (),
        //     }
        // }
    }
}
