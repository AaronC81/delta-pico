use crate::pcf8574::{Pcf8574, Pcf8574Error};
use embedded_hal::blocking::i2c::{Write, Read};

pub struct ButtonMatrix<
    RowI2CDevice: Write<Error = RowError> + Read<Error = RowError>,
    RowError,
    ColI2CDevice: Write<Error = ColError> + Read<Error = ColError>,
    ColError,
> {
    row_pcf: Pcf8574<RowError, RowI2CDevice>,
    col_pcf: Pcf8574<ColError, ColI2CDevice>,
}

impl<
    RowI2CDevice: Write<Error = RowError> + Read<Error = RowError>,
    RowError,
    ColI2CDevice: Write<Error = ColError> + Read<Error = ColError>,
    ColError,
> ButtonMatrix<RowI2CDevice, RowError, ColI2CDevice, ColError> {
    const ROWS: u8 = 7;
    const COLS: u8 = 7;

    // The row/col wiring doesn't exactly correspond to PCF8574 pin numbers.
    // This array maps a PCF8574 bit to a row/col number.
    const PIN_MAPPING: [u8; 7] = [0, 1, 2, 3, 6, 5, 4];

    pub fn new(row_pcf: Pcf8574<RowError, RowI2CDevice>, col_pcf: Pcf8574<ColError, ColI2CDevice>) -> Self {
        Self { row_pcf, col_pcf }
    }

    /// If a button is pressed at this instant, returns the row and column of the pressed button.
    pub fn get_raw_button(&mut self) -> Result<Option<(u8, u8)>, Pcf8574Error> {
        self.col_pcf.write(0xFF)?;

        for row in 0..Self::ROWS {
            // Set all bits except this row
            let row_value = !(1 << row);
            self.row_pcf.write(row_value).unwrap();

            // Check if any buttons in this row were pressed
            let mut byte = self.col_pcf.read().unwrap();
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
}
