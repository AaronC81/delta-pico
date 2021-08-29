use alloc::{string::{String, ToString}, vec};
use rbop::{UnstructuredNodeList, nav::NavPath, node::unstructured::{UnstructuredNodeRoot, Upgradable}, render::{Area, Renderer, Viewport}};

use crate::{interface::ButtonInput, operating_system::os, rbop_impl::{RbopContext, PADDING}};
use super::{Application, ApplicationInfo};
use crate::interface::framework;

pub struct BootloaderApplication {}

impl Application for BootloaderApplication {
    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "Bootloader".into(),
            visible: true,
        }
    }

    fn new() -> Self where Self: Sized { Self {} }

    fn tick(&mut self) {
        os().reboot_into_bootloader();
    }
}
