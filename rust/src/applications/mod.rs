use alloc::{boxed::Box, string::String, vec, vec::Vec};

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

        let mut icon_name = self.name.to_lowercase().replace(" ", "_");
        if icon_name.chars().next().unwrap().is_digit(10) {
            icon_name.insert(0, '_');
        }
        icon_name += "_icon";

        icon_name
    }
}

pub trait Application {
    fn info() -> ApplicationInfo where Self: Sized;

    fn new() -> Self where Self: Sized;
    fn tick(&mut self);

    fn new_dyn() -> Box<dyn Application> where Self: Sized, Self: 'static {
        Box::new(Self::new())
    }

    fn destroy(&mut self) {}
}

pub struct ApplicationList {
    pub applications: Vec<(ApplicationInfo, fn() -> Box<dyn Application>)>,
}

impl ApplicationList {
    pub fn new() -> Self {
        Self {
            applications: vec![],
        }
    }

    pub fn add<T>(&mut self) where T: Application, T: 'static {
        let info = T::info();
        self.applications.push((info, T::new_dyn))
    }
}

#[macro_use]
pub mod real_time;

pub mod menu;
pub mod calculator;
pub mod about;
pub mod graph;
pub mod bootloader;
pub mod storage;
pub mod numbers_game;
pub mod tetris;
pub mod multi_tap;
