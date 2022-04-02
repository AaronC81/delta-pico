use alloc::{vec, vec::Vec};

use crate::{interface::{Colour, ApplicationFramework, DisplayInterface, ButtonInput}, operating_system::{OSInput, UIMenu, UIMenuItem, OperatingSystem}};
use super::{Application, ApplicationInfo};

pub struct MenuApplication<F: ApplicationFramework + 'static> {
    os: *mut OperatingSystem<F>,
    menu: UIMenu<F>,
}

impl<F: ApplicationFramework> Application for MenuApplication<F> {
    type Framework = F;

    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "Menu".into(),
            visible: false,
        }
    }

    fn new(os: *mut OperatingSystem<F>) -> Self where Self: Sized {
        Self {
            os,
            menu: UIMenu::new(os, vec![]),
        }
    }

    fn tick(&mut self) {
        self.os_mut().framework.display_mut().fill_screen(Colour::BLACK);
        self.os_mut().ui_draw_title("Menu");

        // Doesn't work to assign during `new` for some reason, so do this instead
        self.menu.items = self.os().application_list.applications
            .iter()
            .map(|(app, _)| UIMenuItem {
                title: app.name.clone(),
                icon: app.icon_name(),
                toggle: None,
            })
            .collect::<Vec<_>>();
        self.menu.draw();
        self.os_mut().framework.display_mut().draw();

        if let Some(OSInput::Button(btn)) = self.os_mut().input() {
            match btn {
                ButtonInput::MoveUp => self.menu.move_up(),
                ButtonInput::MoveDown => self.menu.move_down(),
                ButtonInput::Exe => self.os_mut().launch_application(self.menu.selected_index),
                _ => (),
            }
        }
    }
}

impl<F: ApplicationFramework> MenuApplication<F> {
    fn os(&self) -> &OperatingSystem<F> { unsafe { &*self.os } }
    fn os_mut(&self) -> &mut OperatingSystem<F> { unsafe { &mut *self.os } }
}
