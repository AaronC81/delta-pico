use alloc::{format, vec::Vec, vec};

use crate::{graphics::colour, interface::{self, ButtonInput, framework}};

use super::{Application, ApplicationInfo};

#[derive(Debug, Clone)]
pub struct RealTimeState<T: Clone> {
    pub time_scheduled_events: Vec<(u32, T)>,
    pub key_events: Vec<(ButtonInput, T)>,
}

impl<T: Clone> RealTimeState<T> {
    fn new() -> Self {
        RealTimeState { time_scheduled_events: vec![], key_events: vec![] }
    }
}

impl<T: Clone> Default for RealTimeState<T> {
    fn default() -> Self { Self::new() }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum RealTimeResult {
    None,
    Redraw,
}

pub trait RealTimeApplication {
    type RealTimeEvent : Clone;

    fn info() -> ApplicationInfo where Self: Sized;
    fn new() -> Self where Self: Sized;

    fn get_real_time_state(&self) -> &RealTimeState<Self::RealTimeEvent>;
    fn get_real_time_state_mut(&mut self) -> &mut RealTimeState<Self::RealTimeEvent>;

    fn on_event(&mut self, event: &Self::RealTimeEvent) -> RealTimeResult;
    fn draw(&mut self);

    fn destroy(&mut self) {}

    fn on_input(&mut self, key: ButtonInput, event: Self::RealTimeEvent) -> &mut Self {
        self.get_real_time_state_mut().key_events.push((key, event));
        self
    }

    fn schedule(&mut self, in_millis: u32, event: Self::RealTimeEvent) -> &mut Self {
        self.get_real_time_state_mut().time_scheduled_events.push(((framework().millis)() + in_millis, event));
        self
    }
}

impl<T: RealTimeApplication> Application for T {
    fn info() -> ApplicationInfo where Self: Sized { Self::info() }
    fn new() -> Self where Self: Sized { Self::new() }

    fn tick(&mut self) {
        // TODO: unnecessarly expensive with all the clones and collects

        let mut results = vec![];

        // Check if there are timed events to fire
        let current_time = (framework().millis)();

        // Fire events whose time has come or passed
        for (event_time, event) in self.get_real_time_state().time_scheduled_events.iter().cloned().collect::<Vec<_>>() {
            if current_time >= event_time {
                results.push(self.on_event(&event));
            }
        }
    
        // In a second pass, remove the events who will have just been fired
        self.get_real_time_state_mut().time_scheduled_events.retain(|(event_time, _)| event_time > &current_time);

        // Fire any key events
        if let Some(key_event) = framework().buttons.immediate_press() {
            for (this_key, event) in self.get_real_time_state().key_events.iter().cloned().collect::<Vec<_>>() {
                if this_key == key_event {
                    results.push(self.on_event(&event));
                }
            }
        }

        // Redraw if an event called for it
        if results.contains(&RealTimeResult::Redraw) {
            self.draw();
        }
    }
}

macro_rules! real_time_boilerplate {
    ($event_type: ident) => {
        type RealTimeEvent = $event_type;

        fn get_real_time_state(&self) -> &RealTimeState<Self::RealTimeEvent> { &self.real_time_state }
        fn get_real_time_state_mut(&mut self) -> &mut RealTimeState<Self::RealTimeEvent> { &mut self.real_time_state }    
    };
}

#[derive(Default)]
pub struct RealTimeTestApplication {
    real_time_state: RealTimeState<RealTimeTestApplicationEvent>,
    second_counter: u32,
    exe_counter: u32,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum RealTimeTestApplicationEvent {
    Start,
    Second,
    Exe,
}

impl RealTimeApplication for RealTimeTestApplication {
    real_time_boilerplate!(RealTimeTestApplicationEvent);

    fn info() -> ApplicationInfo where Self: Sized {
        ApplicationInfo { name: "Real Time Test".into(), visible: false }
    }

    fn new() -> Self where Self: Sized {
        let mut new = Self::default();
        new
            .schedule(0, RealTimeTestApplicationEvent::Start)
            .schedule(1000, RealTimeTestApplicationEvent::Second)
            .on_input(ButtonInput::Exe, RealTimeTestApplicationEvent::Exe);

        new
    }

    fn on_event(&mut self, event: &Self::RealTimeEvent) -> RealTimeResult {
        match event {
            RealTimeTestApplicationEvent::Start => RealTimeResult::Redraw,
            RealTimeTestApplicationEvent::Second => {
                self.second_counter += 1;
                self.schedule(1000, RealTimeTestApplicationEvent::Second);
                RealTimeResult::Redraw
            },
            RealTimeTestApplicationEvent::Exe => {
                self.exe_counter += 1;
                RealTimeResult::Redraw
            },
        }
    }

    fn draw(&mut self) {
        (framework().display.fill_screen)(colour::BLACK);
        framework().display.print_at(50, 50, format!("{} {}", self.second_counter, self.exe_counter));
        (framework().display.draw)();
    }
}
