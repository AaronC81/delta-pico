use alloc::{string::{String, ToString}, vec};
use rbop::{UnstructuredNodeList, nav::NavPath, node::unstructured::{UnstructuredNodeRoot, Upgradable}, render::{Area, Renderer, Viewport}};

use crate::{interface::ButtonInput, operating_system::os, rbop_impl::{RbopContext, PADDING}};
use super::{Application, ApplicationInfo};
use crate::interface::framework;

pub struct MenuApplication {}

impl Application for MenuApplication {
    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "Menu".into(),
            visible: false,
        }
    }

    fn new() -> Self where Self: Sized { Self {} }

    fn tick(&mut self) {
        (framework().display.fill_screen)(0);
        (framework().display.set_cursor)(0, 0);
        framework().display.print("1: Calculator\n".into());
        framework().display.print("2: Bootloader\n".into());
        framework().display.print("\n".into());
        framework().display.print("MENU: Close\n".into());

        (framework().display.draw)();

        if let Some(btn) = framework().buttons.poll_press() {
            match btn {
                ButtonInput::Digit1 => os().launch_application(0),
                ButtonInput::Digit2 => os().reboot_into_bootloader(),
                _ => (),
            }
        }
    }
}
