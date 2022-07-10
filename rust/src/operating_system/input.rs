use crate::interface::{ButtonInput, ApplicationFramework, ButtonsInterface, ButtonEvent};

use super::OperatingSystem;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum OSInput {
    Button(ButtonInput),
    ShiftedButton(ButtonInput),
    TextMultiTapNew(char),
    TextMultiTapCycle(char),
}

impl<F: ApplicationFramework + 'static> OperatingSystem<F> {
    /// Utility method to translate a `ButtonInput` to an `OSInput`.
    /// 
    /// This may have a variety of side effects, including opening/closing menus or changing
    /// multitap state. As such, it should be called only for a *press* and not a release.
    fn button_input_to_os_input(&mut self, input: ButtonInput) -> Option<OSInput> {
        let mut result = match input {
            // Special cases
            ButtonInput::Menu => {
                self.toggle_menu();
                return Some(OSInput::Button(ButtonInput::Menu))
            }
            ButtonInput::Text => {
                self.text_mode = !self.text_mode;
                return None
            }
            ButtonInput::Shift => {
                self.input_shift = !self.input_shift;
                return None
            }
            ButtonInput::None => return None,

            btn if self.input_shift => Some(OSInput::ShiftedButton(btn)),
            btn => Some(OSInput::Button(btn)),
        };

        self.input_shift = false;

        // Intercept if a digit was pressed in text mode - this needs to be converted to a
        // character according to the OS' multi-tap state
        if self.text_mode {
            if let Some(r@OSInput::Button(ButtonInput::Digit(_)) | r@OSInput::ShiftedButton(ButtonInput::Digit(_))) = result {
                result = self.multi_tap.input(r);
            } else {
                // Make sure we don't cycle the wrong character if we e.g. move with the arrows
                self.multi_tap.drop_keypress();
            }
        }

        result
    }

    /// Waits for a key to be pressed, and returns it as `Some(OSInput)`. Can return `None`,
    /// meaning that there is no input but some other event has occurred which requires the
    /// application to tick and redraw.
    /// 
    /// Alternatively, if virtual presses have been queued with `queue_virtual_presses` as part of a
    /// test, pops the queue and returns the next one.
    pub fn input(&mut self) -> Option<OSInput> {
        if let Some(input) = self.virtual_input_queue.get(0).cloned() {
            self.virtual_input_queue.remove(0);
            return input;
        }

        loop {
            let event = self.framework.buttons_mut().wait_event();
            if let ButtonEvent::Press(btn_input) = event {
                self.last_input_millis = self.framework.millis();
                return self.button_input_to_os_input(btn_input)
            }
        }
    }

    /// Queues a sequence of presses to return for subsequent calls to `input`. Each given input is
    /// interspersed with `input` returning `None`. Designed for use when writing tests.
    pub fn queue_virtual_presses(&mut self, buttons: &[OSInput]) {
        for input in buttons.iter().cloned() {
            self.virtual_input_queue.push(Some(input));
            self.virtual_input_queue.push(None);
        }
    }
}
