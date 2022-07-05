use alloc::vec;
use embedded_hal::blocking::{i2c::{Write, Read}, delay::DelayMs};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cat24CError {
    I2C,
}

pub struct Cat24C<I2CDevice: Write<Error = E> + Read<Error = E> + 'static, E, Delay: DelayMs<u8> + 'static> {
    address: u8,
    i2c: &'static mut I2CDevice,
    delay: &'static mut Delay,
}

impl<E, I2CDevice: Write<Error = E> + Read<Error = E> + 'static, Delay: DelayMs<u8> + 'static> Cat24C<I2CDevice, E, Delay> {
    pub fn new(
        address: u8,
        i2c: &'static mut I2CDevice,
        delay: &'static mut Delay,
    ) -> Self {
        Cat24C { address, i2c, delay }
    }

    #[allow(clippy::wrong_self_convention)] // &mut self required for SPI transmission
    pub fn is_connected(&mut self) -> bool {
        // TODO: Untested - I didn't have a Rev.1 to hand while writing this
        let mut buffer = [0; 1];
        self.i2c.read(self.address, &mut buffer[..]).is_ok()
    }

    #[allow(clippy::wrong_self_convention)] // &mut self required for SPI transmission
    pub fn is_busy(&mut self) -> bool {
        // When busy, the device essentially falls off the bus
        !self.is_connected()
    }

    pub fn read(&mut self, address: u16, bytes: &mut [u8]) -> Result<(), Cat24CError> {
        // Write the address we'd like to read from
        self.i2c.write(self.address, &[(address >> 8) as u8, (address & 0xFF) as u8])
            .map_err(|_| Cat24CError::I2C)?;

        // Read the desired number of bytes
        self.i2c.read(self.address, bytes).map_err(|_| Cat24CError::I2C)?;

        Ok(())
    }

    pub fn write(&mut self, address: u16, bytes: &[u8]) -> Result<(), Cat24CError> {
        const PAGE_SIZE: usize = 64;

        // Adapted from Qwiic EEPROM Arduino library
        // Break the buffer into page sized chunks
        let mut recorded = 0;
        while recorded < bytes.len()
        {
            // Limit the amount to write to the page size
            let mut amt_to_write = bytes.len() - recorded;

            if amt_to_write > 1
            {
                //Check for crossing of a page line. Writes cannot cross a page line.
                let page_number_1 = (address as usize + recorded) / PAGE_SIZE;
                let page_number_2 = (address as usize + recorded + amt_to_write - 1) / PAGE_SIZE;
                if page_number_2 > page_number_1 {
                    //Limit the read amt to go right up to edge of page barrier
                    amt_to_write = (page_number_2 * PAGE_SIZE) - (address as usize + recorded);
                }
            }

            // See if EEPROM is available or still writing a previous request
            while self.is_busy() {
                self.delay.delay_ms(1);
            }

            let mut write_buffer = vec![0; bytes.len() + 2];
            write_buffer[0] = ((address as usize + recorded) >> 8) as u8;
            write_buffer[1] = ((address as usize + recorded) & 0xFF) as u8;
            write_buffer[2..].copy_from_slice(bytes);

            self.i2c.write(self.address, &write_buffer[..])
                .map_err(|_| Cat24CError::I2C)?;

            recorded += amt_to_write;

            self.delay.delay_ms(5);
        }

        Ok(())
    }
}

