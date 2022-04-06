use crate::pcf8574::{Pcf8574, Pcf8574Error};
use embedded_hal::blocking::{i2c::{Write, Read}, delay::DelayMs};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RawButtonEvent {
    Press(u8, u8),
    Release(u8, u8),
}

pub struct ButtonMatrix<
    RowI2CDevice: Write<Error = RowError> + Read<Error = RowError>,
    RowError,
    ColI2CDevice: Write<Error = ColError> + Read<Error = ColError>,
    ColError,
    Delay: DelayMs<u8> + 'static,
> {
    row_pcf: Pcf8574<RowError, RowI2CDevice>,
    col_pcf: Pcf8574<ColError, ColI2CDevice>,
    delay: &'static mut Delay,

    currently_pressed: Option<(u8, u8)>,
}

impl<
    RowI2CDevice: Write<Error = RowError> + Read<Error = RowError>,
    RowError,
    ColI2CDevice: Write<Error = ColError> + Read<Error = ColError>,
    ColError,
    Delay: DelayMs<u8>,
> ButtonMatrix<RowI2CDevice, RowError, ColI2CDevice, ColError, Delay> {
    const ROWS: u8 = 7;
    const DEBOUNCE_MS: u8 = 20;

    // The row/col wiring doesn't exactly correspond to PCF8574 pin numbers.
    // This array maps a PCF8574 bit to a row/col number.
    const PIN_MAPPING: [u8; 7] = [0, 1, 2, 3, 6, 5, 4];

    pub fn new(row_pcf: Pcf8574<RowError, RowI2CDevice>, col_pcf: Pcf8574<ColError, ColI2CDevice>, delay: &'static mut Delay) -> Self {
        Self { row_pcf, col_pcf, delay, currently_pressed: None }
    }

    /// If a button is pressed at this instant, returns the row and column of the pressed button.
    pub fn get_raw_button(&mut self) -> Result<Option<(u8, u8)>, Pcf8574Error> {
        self.col_pcf.write(0xFF)?;

        for row in 0..Self::ROWS {
            // Set all bits except this row
            let row_value = !(1 << row);
            self.row_pcf.write(row_value)?;

            // Check if any buttons in this row were pressed
            let mut byte = self.col_pcf.read()?;
            byte = !byte;
            if byte > 0 {
                // Yes! Log2 to find out which col it is
                let mut pressed_col = 0;
                loop {
                    byte >>= 1;
                    if byte == 0 { break; }
                    pressed_col += 1
                };

                // Return the row too
                let pressed_row = row;

                // Map row and column to actual numbers, rather than PCF8574 wiring, and return
                return Ok(Some((
                    Self::PIN_MAPPING[pressed_row as usize],
                    Self::PIN_MAPPING[pressed_col as usize],
                )))
            }
        }

        Ok(None)
    }

    pub fn get_event(&mut self, wait: bool) -> Result<Option<RawButtonEvent>, Pcf8574Error> {
        // Was a button already being pressed?
        if let Some((row, col)) = self.currently_pressed {
            match self.get_raw_button()? {
                // Is it no longer pressed?
                None => {
                    // Is it still no longer pressed after the debounce time?
                    self.delay.delay_ms(Self::DEBOUNCE_MS);
                    if self.get_raw_button()?.is_none() {
                        // The button has been released!
                        self.currently_pressed = None;
                        return Ok(Some(RawButtonEvent::Release(row, col)));
                    }
                }

                // Are we now pressing a different button instead?
                Some((new_row, new_col)) if new_row != row || new_col != col => {
                    // Fire a release now, and let the next iteration catch the new press
                    self.currently_pressed = None;
                    return Ok(Some(RawButtonEvent::Release(row, col)));
                }

                // Same button still being held
                Some((_, _)) => return Ok(None),
            }
        }
    
        let btn = if wait {
            // Wait for a button to be pressed
            loop {
                if let Some(btn) = self.get_raw_button()? {
                    break btn;
                }
            }
        } else {
            // Check immediately
            if let Some(btn) = self.get_raw_button()? {
                btn
            } else {
                return Ok(None);
            }
        };
    
        // Is it still pressed after the debounce time?
        self.delay.delay_ms(Self::DEBOUNCE_MS);
        if let Some(now_btn) = self.get_raw_button()? {
            if btn == now_btn {
                self.currently_pressed = Some(btn);
                let (row, col) = btn;
                return Ok(Some(RawButtonEvent::Press(row, col)));
            }
        }
    
        // Nothing happened
        Ok(None)
    }
}
