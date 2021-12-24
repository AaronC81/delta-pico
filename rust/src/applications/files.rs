use alloc::{format, vec};
use fatfs::{FileSystem, FsOptions, NullTimeProvider, LossyOemCpConverter, Seek, SeekFrom};
use rust_decimal::prelude::ToPrimitive;

use crate::{interface::Colour, operating_system::{OSInput, os, UIMenu, UIMenuItem}, filesystem::FatInterface};
use super::{Application, ApplicationInfo};
use crate::interface::framework;

pub struct FilesApplication {
    menu: UIMenu,
    fat_fs: FileSystem<&'static mut FatInterface<'static>, NullTimeProvider, LossyOemCpConverter>,
}

impl Application for FilesApplication {
    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "Files".into(),
            visible: true,
        }
    }

    fn new() -> Self where Self: Sized {
        // The fatfs library does not seek back to the beginning for us, if this app is opened
        // multiple times after boot
        os().filesystem.fat.seek(SeekFrom::Start(0)).unwrap();

        let fat_fs = FileSystem::new(
            &mut os().filesystem.fat,
            FsOptions::new(),
        ).unwrap();

        let vec =
            fat_fs.root_dir()
                .iter()
                .map(|x| x.unwrap().short_file_name_as_bytes().to_vec())
                .map(|n| UIMenuItem {
                    title: core::str::from_utf8(&n).unwrap().into(),
                    icon: "".into(),
                    toggle: None
                })
                .collect();

        Self {
            menu: UIMenu::new(vec),
            fat_fs,
        }
    }

    fn tick(&mut self) {
        framework().display.fill_screen(Colour::BLACK);

        os().ui_draw_title("Files");

        self.menu.draw();

        framework().display.draw();

        if let Some(btn) = framework().buttons.wait_press() {
            match btn {
                OSInput::MoveUp => self.menu.move_up(),
                OSInput::MoveDown => self.menu.move_down(),
                OSInput::Exe => os().launch_application(self.menu.selected_index),
                _ => (),
            }
        }
    }
}
