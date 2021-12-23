use core::{cell::RefCell, fmt::Display};

use alloc::{rc::Rc, string::String, vec, vec::Vec};

use crate::interface::framework;

#[derive(Debug)]
pub struct Timer {
    pub name: String,
    pub elapsed: u32,
    current_start: Option<u32>,
    subtimers: Vec<Rc<RefCell<Timer>>>,
}

impl Timer {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            elapsed: 0,
            current_start: None,
            subtimers: vec![],
        }
    }

    pub fn micros() -> u32 {
        (framework().micros)()
    }

    pub fn start(&mut self) {
        if self.current_start.is_some() {
            panic!("timer already running")
        }
        self.current_start = Some(Self::micros())
    }

    pub fn stop(&mut self) {
        if self.current_start.is_none() {
            panic!("timer not running")
        }
        let difference = Self::micros() - self.current_start.unwrap();
        self.elapsed += difference;
        self.current_start = None;
    }

    pub fn new_subtimer<'a>(&'a mut self, name: &str) -> Rc<RefCell<Self>> {
        self.subtimers.push(Rc::new(RefCell::new(Self::new(name))));
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

impl Display for Timer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.fmt_with_indent_level(f, 0)
    }
}
