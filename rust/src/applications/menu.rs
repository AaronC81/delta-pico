use alloc::{vec, vec::Vec};

use crate::{interface::{Colour, ApplicationFramework, ButtonInput}, operating_system::{OSInput, FullPageMenu, FullPageMenuItem, OperatingSystem, os_accessor, OperatingSystemPointer}};
use super::{Application, ApplicationInfo};

pub struct MenuApplication<F: ApplicationFramework + 'static> {
    os: OperatingSystemPointer<F>,
    menu: FullPageMenu<F>,
}

os_accessor!(MenuApplication<F>);

impl<F: ApplicationFramework> Application for MenuApplication<F> {
    type Framework = F;

    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "Menu".into(),
            visible: false,
        }
    }

    fn new(os: OperatingSystemPointer<F>) -> Self where Self: Sized {
        Self {
            os,
            menu: FullPageMenu::new(os, vec![]),
        }
    }

    fn tick(&mut self) {
        self.os_mut().display_sprite.fill(Colour::BLACK);
        self.os_mut().ui_draw_title("Menu");

        // Doesn't work to assign during `new` for some reason, so do this instead
        self.menu.items = self.os().application_list.applications
            .iter()
            .map(|(app, _)| FullPageMenuItem {
                title: app.name.clone(),
                icon: app.icon_name(),
                toggle: None,
            })
            .collect::<Vec<_>>();
        self.menu.draw();
        self.os_mut().draw();

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
