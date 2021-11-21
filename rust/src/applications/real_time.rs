use alloc::{format, vec::Vec, vec};

use crate::{interface::{self, ButtonInput, Colour, framework}, operating_system::OSInput};

use super::{Application, ApplicationInfo};

#[derive(Debug, Clone)]
/// The internal state of a struct implementing `RealTimeApplication`.
pub struct RealTimeState<T: Clone> {
    /// Events scheduled for a particular time in milliseconds.
    pub time_scheduled_events: Vec<(u32, T)>,

    /// Events to run when a button is pressed.
    pub key_events: Vec<(OSInput, T)>,
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
/// What to do after a real-time event is handled.
pub enum RealTimeResult {
    /// Nothing.
    None,

    /// Redraw the screen by calling `RealTimeApplication::draw`.
    Redraw,
}

/// A trait for creating applications which do not block on user input, instead using an
/// event-driven model to fire events at particular times or when certain keys are pressed.
/// 
/// Despite the name, these applications are not truly "real-time", and do not make timing
/// guarantees. Try to avoid writing `on_event` implementations which block for significant periods
/// of time, and it will feel rather similar to truly real-time.
/// 
/// Implementing `RealTimeApplication` will automatically implement `Application`.
/// `Application::tick` is implemented as a dispatcher for time-based events, and also polls for
/// input, running in a loop with no delay between iterations.
/// 
/// Keypresses for special keys like MENU are handled automatically.
pub trait RealTimeApplication {
    /// The event type to use within this application. Typically an enum of all of the different
    /// events which can occur.
    type RealTimeEvent : Clone;

    fn info() -> ApplicationInfo where Self: Sized;
    fn new() -> Self where Self: Sized;

    fn get_real_time_state(&self) -> &RealTimeState<Self::RealTimeEvent>;
    fn get_real_time_state_mut(&mut self) -> &mut RealTimeState<Self::RealTimeEvent>;

    /// Fired when an event registerd using `on_input` or `schedule` occurs.
    fn on_event(&mut self, event: &Self::RealTimeEvent) -> RealTimeResult;

    /// Draw to the display. Will be called if any `on_event` during a tick returns
    /// `RealTimeResult::Redraw`.
    fn draw(&mut self);

    fn destroy(&mut self) {}

    /// Register an `event` to be fired whenever a particular `key` is pressed. This event is
    /// permanent and will fire for every press - that is, it isn't removed after the key is pressed
    /// once. If you do need to remove this handler for some reason, you can find the corresponding
    /// entry in the `key_events` field on the implementor's `RealTimeState` and delete it.
    fn on_input(&mut self, key: OSInput, event: Self::RealTimeEvent) -> &mut Self {
        self.get_real_time_state_mut().key_events.push((key, event));
        self
    }

    /// Register an `event` to be fired `in_millis` milliseconds from now. Since that time in
    /// milliseconds will only occur once, the handler is removed once the event has been handled.
    /// If you need to perform an event repeatedly, you can set up the event handler to call
    /// `schedule` again.
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
            .on_input(OSInput::Exe, RealTimeTestApplicationEvent::Exe);

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
        framework().display.fill_screen(Colour::BLACK);
        framework().display.print_at(50, 50, &format!("{} {}", self.second_counter, self.exe_counter));
        (framework().display.draw)();
    }
}
