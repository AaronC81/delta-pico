use alloc::{format, string::{String, ToString}, vec};
use rbop::{UnstructuredNodeList, nav::NavPath, node::unstructured::{UnstructuredNodeRoot, Upgradable}, render::{Area, Renderer, Viewport}};

use crate::{interface::ButtonInput, operating_system::os, rbop_impl::{RbopContext}};
use super::{Application, ApplicationInfo};
use crate::interface::framework;
use crate::graphics::colour;

pub struct AboutApplication {}

impl Application for AboutApplication {
    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "About".into(),
            visible: true,
        }
    }

    fn new() -> Self where Self: Sized { Self {} }

    fn tick(&mut self) {
        (framework().display.fill_screen)(colour::BLACK);
        (framework().display.set_cursor)(60, 50);

        framework().display.print("DELTA PICO\n\n\n");

        framework().display.print("Software version:\n");
        framework().display.print(format!("    {}\n", env!("CARGO_PKG_VERSION")));
        framework().display.print(format!("    rev {}\n\n", env!("GIT_VERSION")));

        framework().display.print("rbop version:\n");
        framework().display.print(format!("    {}\n\n", rbop::VERSION));

        (framework().display.draw)();

        framework().buttons.poll_press();
    }
}
