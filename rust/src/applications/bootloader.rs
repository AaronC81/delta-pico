use crate::operating_system::os;
use super::{Application, ApplicationInfo};

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
