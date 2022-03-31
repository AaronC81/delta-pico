use cortex_m::prelude::{_embedded_hal_blocking_i2c_Write, _embedded_hal_blocking_i2c_Read};
use embedded_hal::blocking::i2c::{Write, Read};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pcf8574Error {
    I2CError,
}

pub struct Pcf8574<E, I2CDevice: Write<Error = E> + Read<Error = E>> {
    address: u8,
    i2c: I2CDevice,
}

impl<E, I2CDevice: Write<Error = E> + Read<Error = E>> Pcf8574<E, I2CDevice> {
    pub fn new(
        address: u8,
        i2c: I2CDevice,
    ) -> Self {
        Pcf8574 { address, i2c }
    }

    pub fn write(&mut self, value: u8) -> Result<(), Pcf8574Error> {
        self.i2c.write(self.address, &[value]).map_err(|_| Pcf8574Error::I2CError)
    }

    pub fn read(&mut self) -> Result<u8, Pcf8574Error> {
        let mut buffer = [0u8; 1];
        self.i2c.read(self.address, &mut buffer).map_err(|_| Pcf8574Error::I2CError)?;
        Ok(buffer[0])
    }
}

