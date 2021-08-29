use alloc::{string::{String, ToString}, vec};
use rbop::{UnstructuredNodeList, nav::NavPath, node::unstructured::{UnstructuredNodeRoot, Upgradable}, render::{Area, Renderer, Viewport}};

use crate::{operating_system::os, rbop_impl::{RbopContext, PADDING}};
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
        (framework().display.set_cursor)(0, 0);
        framework().display.print("Press any key".into());

        (framework().display.draw)();

        if framework().buttons.poll_press().is_some() {
            os().launch_application(0);
        }
    }
}
