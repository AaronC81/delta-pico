use alloc::{boxed::Box, string::String, vec, vec::Vec};
use crate::{operating_system::OperatingSystem, interface::ApplicationFramework};

pub struct ApplicationInfo {
    pub name: String,
    pub visible: bool,
}

impl ApplicationInfo {
    fn icon_name(&self) -> String {
        // Icon names must be C identifiers, so:
        //   - Replace spaces with underscores
        //   - Prefix with an underscore if the name begins with a digit
        // They are also suffixed with _icon

        let mut icon_name = self.name.to_lowercase().replace(' ', "_");
        if icon_name.chars().next().unwrap().is_digit(10) {
            icon_name.insert(0, '_');
        }
        icon_name += "_icon";

        icon_name
    }
}

pub trait Application {
    type Framework : ApplicationFramework;

    fn info() -> ApplicationInfo where Self: Sized;

    fn new(os: *mut OperatingSystem<Self::Framework>) -> Self where Self: Sized;
    fn tick(&mut self);

    fn new_dyn(os: *mut OperatingSystem<Self::Framework>) -> Box<dyn Application<Framework = Self::Framework>> where Self: Sized, Self: 'static {
        Box::new(Self::new(os))
    }

    fn destroy(&mut self) {}

    fn test_info(&self) -> Vec<String> {
        vec![]
    }
}

pub struct ApplicationList<F: ApplicationFramework + 'static> {
    pub os: *mut OperatingSystem<F>,
    pub applications: Vec<(ApplicationInfo, fn(*mut OperatingSystem<F>) -> Box<dyn Application<Framework = F>>)>,
}

impl<'a, 'b, F: ApplicationFramework> ApplicationList<F> {
    pub fn new() -> Self {
        Self {
            os: core::ptr::null_mut(),
            applications: vec![],
        }
    }

    pub fn add<T>(&mut self) where T: Application<Framework = F> + 'static {
        let info = T::info();
        self.applications.push((info, T::new_dyn))
    }
}

macro_rules! os_accessor {
    ($n:ty) => {
        impl<F: ApplicationFramework> $n {
            #[allow(unused)]
            fn os(&self) -> &OperatingSystem<F> { unsafe { &*self.os } }

            #[allow(unused)]
            fn os_mut(&self) -> &mut OperatingSystem<F> { unsafe { &mut *self.os } }        
        }
    };
}

pub mod menu;
// pub mod calculator;
pub mod about;
// pub mod graph;
pub mod bootloader;
// pub mod storage;
pub mod numbers_game;
// pub mod settings;
// pub mod files;
