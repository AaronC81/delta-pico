use alloc::{boxed::Box, vec};

use crate::applications::{Application, ApplicationList, menu::MenuApplication};

static mut OPERATING_SYSTEM_INTERFACE: Option<OperatingSystemInterface> = None;
pub fn os() -> &'static mut OperatingSystemInterface {
    unsafe {
        if OPERATING_SYSTEM_INTERFACE.is_none() {
            OPERATING_SYSTEM_INTERFACE = Some(OperatingSystemInterface {
                application_list: ApplicationList::new(),
                active_application: None, 
                menu: MenuApplication::new(),
            });
        }
        OPERATING_SYSTEM_INTERFACE.as_mut().unwrap()
    }
}

pub struct OperatingSystemInterface {
    pub application_list: ApplicationList,
    pub menu: MenuApplication,
    pub active_application: Option<Box<dyn Application>>,
}

impl OperatingSystemInterface {
    pub fn launch_application(&mut self, index: usize) {
        self.active_application = Some(self.application_list.applications[index].1());
    }

    pub fn application_to_tick(&mut self) -> &mut dyn Application {
        self.active_application.as_mut()
            .map(|x| x.as_mut())
            .unwrap_or(&mut self.menu)
    }
}
