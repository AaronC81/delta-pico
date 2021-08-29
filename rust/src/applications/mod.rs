use alloc::string::String;

pub trait Application {
    fn name() -> String;
    fn visible() -> bool;

    fn new() -> Self;
    fn tick(&mut self);
}

pub mod calculator;
