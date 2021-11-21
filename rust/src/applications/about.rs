use alloc::{format, string::{String, ToString}, vec};
use rbop::{UnstructuredNodeList, nav::NavPath, node::unstructured::{UnstructuredNodeRoot, Upgradable}, render::{Area, Renderer, Viewport}};

use crate::{interface::{ButtonInput, Colour}, operating_system::os, rbop_impl::{RbopContext}};
use super::{Application, ApplicationInfo};
use crate::interface::framework;

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
        framework().display.fill_screen(Colour::BLACK);

        os().ui_draw_title("About Delta Pico");

        framework().display.print_at(5, 40,  "Software version:");
        framework().display.print_at(5, 60,  &format!("  {}", env!("CARGO_PKG_VERSION")));
        framework().display.print_at(5, 80,  &format!("  rev {}", env!("GIT_VERSION")));
        framework().display.print_at(5, 100, &format!("  rbop {}", rbop::VERSION));

        framework().display.print_at(70, 250,  "Created by");
        framework().display.print_at(35, 270,  "Aaron Christiansen");
        framework().display.print_at(110, 290,  ":)");

        framework().display.draw();

        framework().buttons.wait_press();
    }
}
