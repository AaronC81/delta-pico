use alloc::{format, string::{String, ToString}, vec};
use rbop::{UnstructuredNodeList, nav::NavPath, node::unstructured::{UnstructuredNodeRoot, Upgradable}, render::{Area, Renderer, Viewport}};

use crate::{interface::ButtonInput, operating_system::os, rbop_impl::{RbopContext}};
use super::{Application, ApplicationInfo};
use crate::interface::framework;
use crate::graphics::colour;

const SHOW_BYTES: u16 = 64;

pub struct MemoryApplication {
    address: u16,
}

impl Application for MemoryApplication {
    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "Memory".into(),
            visible: true,
        }
    }

    fn new() -> Self where Self: Sized { Self {
        address: 0
    } }

    fn tick(&mut self) {
        (framework().display.fill_screen)(colour::BLACK);

        os().ui_draw_title("Memory");

        (framework().display.set_cursor)(0, 50);

        framework().display.print(format!("Memory range {}-{}\n\n\n", self.address, self.address + SHOW_BYTES - 1));

        let bytes = framework().storage.read(self.address, SHOW_BYTES as u8).unwrap();
        for i in 0..(SHOW_BYTES / 8) {
            for j in 0..8 {
                framework().display.print(&format!("{:#04x} ", bytes[(i * 8 + j) as usize])[2..=4])
            }
            framework().display.print("\n");
        }
        (framework().display.draw)();

        if let Some(input) = framework().buttons.poll_press() {
            match input {
                ButtonInput::MoveDown => self.address += SHOW_BYTES,
                ButtonInput::MoveUp => self.address -= SHOW_BYTES,
                _ => (),
            }
        };
    }
}
