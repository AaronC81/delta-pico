use crate::{interface::ApplicationFramework, operating_system::{OperatingSystem, os_accessor}};
use super::{Application, ApplicationInfo};

pub struct BootloaderApplication<F: ApplicationFramework + 'static> {
    os: *mut OperatingSystem<F>,
}

os_accessor!(BootloaderApplication<F>);

impl<F: ApplicationFramework> Application for BootloaderApplication<F> {
    type Framework = F;

    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "Bootloader".into(),
            visible: true,
        }
    }

    fn new(os: *mut OperatingSystem<F>) -> Self { Self { os } }

    fn tick(&mut self) {
        self.os_mut().framework.reboot_into_bootloader();
    }
}
