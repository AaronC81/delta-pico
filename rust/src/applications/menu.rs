use alloc::{vec, vec::Vec};

use crate::{interface::{Colour, ApplicationFramework, DisplayInterface}, operating_system::{OSInput, UIMenu, UIMenuItem, OperatingSystem}};
use super::{Application, ApplicationInfo};

pub struct MenuApplication<F: ApplicationFramework + 'static> {
    os: *mut OperatingSystem<F>,
    menu: UIMenu,
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
            menu: UIMenu::new(vec![]),
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

impl<F: ApplicationFramework> MenuApplication<F> {
    fn os(&self) -> &OperatingSystem<F> { unsafe { &*self.os } }
    fn os_mut(&self) -> &mut OperatingSystem<F> { unsafe { &mut *self.os } }
}
