use alloc::{boxed::Box, string::String, vec, vec::Vec};

pub struct ApplicationInfo {
    pub name: String,
    pub visible: bool,
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

pub mod menu;
pub mod calculator;
pub mod about;
pub mod graph;
pub mod bootloader;
pub mod storage;
