use core::{cell::RefCell, fmt::Display};

use alloc::{rc::Rc, string::String, vec, vec::Vec};

use crate::{operating_system::{OperatingSystem, os_accessor}, interface::ApplicationFramework};

#[derive(Debug)]
pub struct Timer<F: ApplicationFramework + 'static> {
    os: *mut OperatingSystem<F>,

    pub name: String,
    pub elapsed: u64,
    current_start: Option<u64>,
    subtimers: Vec<Rc<RefCell<Timer<F>>>>,
}

os_accessor!(Timer<F>);

impl<F: ApplicationFramework> Timer<F> {
    pub fn new(os: *mut OperatingSystem<F>, name: &str) -> Self {
        Self {
            os,
            name: name.into(),
            elapsed: 0,
            current_start: None,
            subtimers: vec![],
        }
    }

    pub fn start(&mut self) {
        if self.current_start.is_some() {
            panic!("timer already running")
        }
        self.current_start = Some(self.os().framework.micros());
    }

    pub fn stop(&mut self) {
        if self.current_start.is_none() {
            panic!("timer not running")
        }
        let difference = self.os().framework.micros() - self.current_start.unwrap();
        self.elapsed += difference;
        self.current_start = None;
    }

    pub fn new_subtimer(&mut self, name: &str) -> Rc<RefCell<Self>> {
        self.subtimers.push(Rc::new(RefCell::new(Self::new(self.os, name))));
        self.subtimers.last().unwrap().clone()
    }

    fn fmt_with_indent_level(&self, f: &mut core::fmt::Formatter<'_>, indent: usize) -> core::fmt::Result {
        let indent_s = "  ".repeat(indent);
        f.write_fmt(format_args!("{}{}: {}\n", indent_s, &self.name, self.elapsed))?;
        for subtimer in &self.subtimers {
            subtimer.borrow().fmt_with_indent_level(f, indent + 1)?;
        }
        core::fmt::Result::Ok(())
    }
}

impl<F: ApplicationFramework> Display for Timer<F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.fmt_with_indent_level(f, 0)
    }
}
