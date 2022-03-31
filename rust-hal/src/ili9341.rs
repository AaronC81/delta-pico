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
        self.send_packet(0x0F, &[0x03, 0x80, 0x02])?;
        self.send_packet(0xCF, &[0x00, 0xC1, 0x30])?;
        self.send_packet(0xed, &[0x64, 0x03, 0x12, 0x81])?;
        self.send_packet(0xe8, &[0x85, 0x00, 0x78])?;
        self.send_packet(0xcb, &[0x39, 0x2c, 0x00, 0x34, 0x02])?;
        self.send_packet(0xf7, &[0x20])?;
        self.send_packet(0xea, &[0x00, 0x00])?;
        self.send_packet(0xc0, &[0x23])?;
        self.send_packet(0xc1, &[0x10])?;
        self.send_packet(0xc5, &[0x3e, 0x28])?;
        self.send_packet(0xc7, &[0x86])?;
        self.send_packet(0x36, &[0x48])?;
        self.send_packet(0x3a, &[0x55])?;
        self.send_packet(0xb1, &[0x00, 0x18])?;
        self.send_packet(0xb6, &[0x08, 0x82, 0x27])?;
        self.send_packet(0xf2, &[0x00])?;
        self.send_packet(0x26, &[0x01])?;
        self.send_packet(0xe0, &[0xf, 0x31, 0x2b, 0xc, 0xe, 0x8, 0x4e, 0xf1, 0x37, 0x7, 0x10, 0x3, 0xe, 0x9, 0x0])?;
        self.send_packet(0xe1, &[0x0, 0xe, 0x14, 0x3, 0x11, 0x7, 0x31, 0xc1, 0x48, 0x8, 0xf, 0xc, 0x31, 0x36, 0xf])?;
        
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

    pub fn send_packet(&mut self, command: u8, data: &[u8]) -> Result<(), Ili9341Error> {
        self.send_command(command)?;
        for byte in data {
            self.send_data(*byte)?;
        }
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
        self.send_packet(0x2A, &[0, 0, 0, 240])?;

        // PASET
        self.send_packet(0x2B, &[0, 0, 0x01, 0x40])?;

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

