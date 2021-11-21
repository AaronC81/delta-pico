use alloc::{format, string::{String, ToString}, vec};
use rbop::{UnstructuredNodeList, nav::NavPath, node::unstructured::{UnstructuredNodeRoot, Upgradable}, render::{Area, Renderer, Viewport}};
use rust_decimal::prelude::ToPrimitive;

use crate::{interface::{ButtonInput, Colour}, operating_system::{OSInput, os}, rbop_impl::{RbopContext}};
use super::{Application, ApplicationInfo};
use crate::interface::framework;

const SHOW_BYTES: u16 = 64;

pub struct StorageApplication {
    address: u16,
}

impl Application for StorageApplication {
    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "Storage".into(),
            visible: true,
        }
    }

    fn new() -> Self where Self: Sized { Self {
        address: 0
    } }

    fn tick(&mut self) {
        framework().display.fill_screen(Colour::BLACK);

        os().ui_draw_title("Storage");

        (framework().display.set_cursor)(0, 50);

        framework().display.print(&format!("Address range {}-{}\n\n\n", self.address, self.address + SHOW_BYTES - 1));

        let bytes = framework().storage.read(self.address, SHOW_BYTES as u8).unwrap();
        for i in 0..(SHOW_BYTES / 8) {
            for j in 0..8 {
                framework().display.print(&format!("{:#04x} ", bytes[(i * 8 + j) as usize])[2..=4])
            }
            framework().display.print("\n");
        }
        (framework().display.draw)();

        if let Some(input) = framework().buttons.wait_press() {
            match input {
                OSInput::MoveDown => self.address += SHOW_BYTES,
                OSInput::MoveUp => self.address -= SHOW_BYTES,
                OSInput::List => {
                    match os().ui_open_menu(&["Jump".into(), "Clear memory".into()], true) {
                        Some(0) => {
                            // TODO redraw
                            let address_dec = os().ui_input_expression_and_evaluate("Memory address", None, || ());
                            if let Some(address) = address_dec.to_decimal().to_u16() {
                                // Bind to boundary
                                self.address = (address / SHOW_BYTES) * SHOW_BYTES;
                            } else {
                                os().ui_text_dialog("Invalid address");
                            }
                        }
                        Some(1) => {
                            if os().filesystem.clear().is_some() {
                                os().ui_text_dialog("Memory cleared.");
                            } else {
                                os().ui_text_dialog("Failed to clear memory.");
                            }
                        },
                        _ => (),
                    }
                }
                _ => (),
            }
        };
    }
}
