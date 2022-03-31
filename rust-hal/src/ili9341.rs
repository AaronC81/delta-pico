use core::marker::PhantomData;
use cortex_m::prelude::{_embedded_hal_spi_FullDuplex};
use embedded_hal::{digital::v2::OutputPin, blocking::delay::DelayMs};
use rp_pico::hal::{Spi, spi::SpiDevice, spi, gpio::{Pin, PinId, Output, PushPull}};
use nb::{self, block};

pub struct Enabled;
pub struct Disabled;
pub trait State {}
impl State for Enabled {}
impl State for Disabled {}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ili9341Error {
    GpioError,
    SpiError,
}

// TODO: would ne cool to implement fast_write using borrow checker
// enforce that we can't call other write_* methods while a "fast writer" exists
// maybe somehow move SPI to a struct whose borrow needs to end before this can be used again?

pub struct Ili9341<
    'a,
    S: State,
    SpiD: SpiDevice,
    DcPin: PinId,
    RstPin: PinId,
    Delay: DelayMs<u8>,
> {
    width: u32,
    height: u32,

    spi: &'a mut Spi<spi::Enabled, SpiD, 8>,
    dc: &'a mut Pin<DcPin, Output<PushPull>>,
    rst: &'a mut Pin<RstPin, Output<PushPull>>,
    delay: &'a mut Delay,

    state: PhantomData<S>,
}

impl<'a, S: State, SpiD: SpiDevice, DcPin: PinId, RstPin: PinId, Delay: DelayMs<u8>> Ili9341<'a, S, SpiD, DcPin, RstPin, Delay> {
    fn change_state<NewS: State>(self) -> Ili9341<'a, NewS, SpiD, DcPin, RstPin, Delay> {
        Ili9341::<'a, NewS, _, _, _, _> {
            width: self.width,
            height: self.height,
            spi: self.spi,
            dc: self.dc,
            rst: self.rst,
            delay: self.delay,
            state: PhantomData,
        }
    }

    pub fn hardware_reset(self) -> Result<Ili9341<'a, Disabled, SpiD, DcPin, RstPin, Delay>, Ili9341Error> {
        self.rst.set_low().map_err(|_| Ili9341Error::GpioError)?;
        self.delay.delay_ms(50);
        self.rst.set_high().map_err(|_| Ili9341Error::GpioError)?;
        self.delay.delay_ms(50);

        Ok(self.change_state::<Disabled>())
    }

    pub fn software_reset(mut self) -> Result<Ili9341<'a, Disabled, SpiD, DcPin, RstPin, Delay>, Ili9341Error> {
        // Unsleep
        self.send_command(0x11)?;
        self.delay.delay_ms(150);

        // Display on
        self.send_command(0x29)?;
        self.delay.delay_ms(150);

        Ok(self.change_state::<Disabled>())
    }

    fn send_init_commands(&mut self) -> Result<(), Ili9341Error> {
        self.send_command(0x0f)?;
        self.send_data(0x03)?; self.send_data(0x80)?; self.send_data(0x02)?;
        self.send_command(0xcf)?;
        self.send_data(0x00)?; self.send_data(0xc1)?; self.send_data(0x30)?;
        self.send_command(0xed)?;
        self.send_data(0x64)?; self.send_data(0x03)?; self.send_data(0x12)?; self.send_data(0x81)?;
        self.send_command(0xe8)?;
        self.send_data(0x85)?; self.send_data(0x00)?; self.send_data(0x78)?;
        self.send_command(0xcb)?;
        self.send_data(0x39)?; self.send_data(0x2c)?; self.send_data(0x00)?; self.send_data(0x34)?; self.send_data(0x02)?;
        self.send_command(0xf7)?;
        self.send_data(0x20)?;
        self.send_command(0xea)?;
        self.send_data(0x00)?; self.send_data(0x00)?;
        self.send_command(0xc0)?;
        self.send_data(0x23)?;
        self.send_command(0xc1)?;
        self.send_data(0x10)?;
        self.send_command(0xc5)?;
        self.send_data(0x3e)?; self.send_data(0x28)?;
        self.send_command(0xc7)?;
        self.send_data(0x86)?;
        
        self.send_command(0x36)?;
        self.send_data(0x48)?;
    
        self.send_command(0x3a)?;
        self.send_data(0x55)?;
        self.send_command(0xb1)?;
        self.send_data(0x00)?; self.send_data(0x18)?;
        self.send_command(0xb6)?;
        self.send_data(0x08)?; self.send_data(0x82)?; self.send_data(0x27)?;
        self.send_command(0xf2)?;
        self.send_data(0x00)?;
        self.send_command(0x26)?;
        self.send_data(0x01)?;
        
        self.send_command(0xe0)?;
        self.send_data(0xf)?; self.send_data(0x31)?; self.send_data(0x2b)?; self.send_data(0xc)?; self.send_data(0xe)?; self.send_data(0x8)?; self.send_data(0x4e)?; self.send_data(0xf1)?; self.send_data(0x37)?; self.send_data(0x7)?; self.send_data(0x10)?; self.send_data(0x3)?; self.send_data(0xe)?; self.send_data(0x9)?; self.send_data(0x0)?;
    
        self.send_command(0xe1)?;
        self.send_data(0x0)?; self.send_data(0xe)?; self.send_data(0x14)?; self.send_data(0x3)?; self.send_data(0x11)?; self.send_data(0x7)?; self.send_data(0x31)?; self.send_data(0xc1)?; self.send_data(0x48)?; self.send_data(0x8)?; self.send_data(0xf)?; self.send_data(0xc)?; self.send_data(0x31)?; self.send_data(0x36)?; self.send_data(0xf)?;

        Ok(())
    }

    pub fn send_command(&mut self, byte: u8) -> Result<(), Ili9341Error> {
        self.dc.set_low().map_err(|_| Ili9341Error::GpioError)?;
        block!(self.spi.send(byte)).map_err(|_| Ili9341Error::SpiError)?;
        Ok(())
    }

    pub fn send_data(&mut self, byte: u8) -> Result<(), Ili9341Error> {
        self.dc.set_high().map_err(|_| Ili9341Error::GpioError)?;
        block!(self.spi.send(byte)).map_err(|_| Ili9341Error::SpiError)?;
        Ok(())
    }
}

impl<'a, SpiD: SpiDevice, DcPin: PinId, RstPin: PinId, Delay: DelayMs<u8>> Ili9341<'a, Disabled, SpiD, DcPin, RstPin, Delay> {
    pub fn new(
        width: u32,
        height: u32,
        spi: &'a mut Spi<spi::Enabled, SpiD, 8>,
        dc: &'a mut Pin<DcPin, Output<PushPull>>,
        rst: &'a mut Pin<RstPin, Output<PushPull>>,
        delay: &'a mut Delay,
    ) -> Ili9341<'a, Disabled, SpiD, DcPin, RstPin, Delay> {
        Ili9341 {
            width,
            height,
            spi,
            dc,
            rst,
            delay,
            state: PhantomData,
        }
    }

    pub fn init(self) -> Result<Ili9341<'a, Enabled, SpiD, DcPin, RstPin, Delay>, Ili9341Error> {
        // Reconstruct as enabled
        let mut result = self.hardware_reset()?;
        result.send_init_commands()?;
        let result = result.software_reset()?;
        
        Ok(result.change_state::<Enabled>())
    }
}

impl<'a, SpiD: SpiDevice, DcPin: PinId, RstPin: PinId, Delay: DelayMs<u8>> Ili9341<'a, Enabled, SpiD, DcPin, RstPin, Delay> {
    pub fn fill_screen(&mut self) -> Result<(), Ili9341Error> {
        // CASET
        self.send_command(0x2A)?;
        self.send_data(0)?;
        self.send_data(0)?;
        self.send_data(0)?;
        self.send_data(240)?;

        // PASET
        self.send_command(0x2B)?;
        self.send_data(0)?;
        self.send_data(0)?;
        self.send_data(0x01)?;
        self.send_data(0x40)?;

        // RAMWR
        self.send_command(0x2C)?;

        // Write bytes
        self.dc.set_high().unwrap();
        for _ in 0..self.width {
            for _ in 0..self.height {
                nb::block!(self.spi.send(0)).unwrap();
                nb::block!(self.spi.send(0)).unwrap();
            }
        }

        Ok(())
    }
}

