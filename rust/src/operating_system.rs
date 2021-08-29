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
                showing_menu: true,
            });
        }
        OPERATING_SYSTEM_INTERFACE.as_mut().unwrap()
    }
}

pub struct OperatingSystemInterface {
    pub application_list: ApplicationList,
    pub menu: MenuApplication,
    pub showing_menu: bool,
    pub active_application: Option<Box<dyn Application>>,
}

impl OperatingSystemInterface {
    pub fn launch_application(&mut self, index: usize) {
        self.showing_menu = false;
        self.active_application = Some(self.application_list.applications[index].1());
    }

    pub fn application_to_tick(&mut self) -> &mut dyn Application {
        if self.showing_menu {
            &mut self.menu
        } else {
            self.active_application.as_mut()
                .map(|x| x.as_mut())
                .unwrap_or(&mut self.menu)
        }
    }

    pub fn toggle_menu(&mut self) {
        self.showing_menu = !self.showing_menu;
    }
}
