use alloc::{format, vec};
use rust_decimal::prelude::ToPrimitive;

use crate::{interface::{Colour, ApplicationFramework, StorageInterface, ButtonInput}, operating_system::{OSInput, OperatingSystem, os_accessor, OperatingSystemPointer, ContextMenu, ContextMenuItem, SelectorMenuCallable}};
use super::{Application, ApplicationInfo};

const SHOW_BYTES: u16 = 64;

pub struct StorageApplication<F: ApplicationFramework + 'static> {
    os: OperatingSystemPointer<F>,
    address: u16,
}

os_accessor!(StorageApplication<F>);

impl<F: ApplicationFramework> Application for StorageApplication<F> {
    type Framework = F;

    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "Raw Storage".into(),
            visible: true,
        }
    }

    fn new(os: OperatingSystemPointer<F>) -> Self {
        Self { os, address: 0 }
    }

    fn tick(&mut self) {
        self.os_mut().display_sprite.fill(Colour::BLACK);
        self.os_mut().ui_draw_title("Raw Storage");

        self.os_mut().display_sprite.print_at(0, 50, &format!(
            "Address range {}-{}\n\n\n", self.address, self.address + SHOW_BYTES - 1
        ));


        let mut bytes = [0; SHOW_BYTES as usize];
        self.os_mut().framework.storage_mut().read(self.address, &mut bytes[..]).unwrap();
        for i in 0..(SHOW_BYTES / 8) {
            for j in 0..8 {
                self.os_mut().display_sprite.print(&format!("{:#04x} ", bytes[(i * 8 + j) as usize])[2..=4])
            }
            self.os_mut().display_sprite.print("\n");
        }
        self.os_mut().draw();

        if let Some(input) = self.os_mut().input() {
            match input {
                OSInput::Button(ButtonInput::MoveDown) => self.address += SHOW_BYTES,
                OSInput::Button(ButtonInput::MoveUp) => self.address -= SHOW_BYTES,
                OSInput::Button(ButtonInput::List) => {
                    ContextMenu::new(
                        self.os,
                        vec![
                            ContextMenuItem::new_common("Jump...", |this: &mut Self| {
                                // TODO redraw
                                let (address_dec, _) = this.os_mut().ui_input_expression_and_evaluate("Memory address", None, || ());
                                if let Some(address) = address_dec.to_decimal().to_u16() {
                                    // Bind to boundary
                                    this.address = (address / SHOW_BYTES) * SHOW_BYTES;
                                } else {
                                    this.os_mut().ui_text_dialog("Invalid address");
                                }
                            })
                        ],
                        true,
                    ).tick_until_call(self);
                }
                _ => (),
            }
        };
    }
}
