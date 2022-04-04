use core::marker::PhantomData;
use cortex_m::prelude::{_embedded_hal_spi_FullDuplex};
use delta_pico_rust::{interface::Colour, graphics::Sprite};
use embedded_hal::{digital::v2::OutputPin, blocking::delay::DelayMs};
use rp_pico::hal::{Spi, spi::SpiDevice, spi, gpio::{Pin, PinId, Output, PushPull}};
use nb::{self, block};

use crate::util::saturating_into::SaturatingInto;

pub struct Enabled;
pub struct Disabled;
pub trait State {}
impl State for Enabled {}
impl State for Disabled {}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ili9341Error {
    GpioError,
    SpiError,
    BoundsError,
}

pub struct Ili9341<
    S: State,
    SpiD: SpiDevice,
    DcPin: PinId,
    RstPin: PinId,
    Delay: DelayMs<u8>,
> {
    pub width: u16,
    pub height: u16,

    spi: Spi<spi::Enabled, SpiD, 8>,
    dc: Pin<DcPin, Output<PushPull>>,
    rst: Pin<RstPin, Output<PushPull>>,
    delay: Delay,

    state: PhantomData<S>,
}

pub struct Ili9341FastDataWriter<
    'a,
    SpiD: SpiDevice,
    DcPin: PinId,
    RstPin: PinId,
    Delay: DelayMs<u8>,
> {
    ili9341: &'a mut Ili9341<Enabled, SpiD, DcPin, RstPin, Delay>,
}

impl<S: State, SpiD: SpiDevice, DcPin: PinId, RstPin: PinId, Delay: DelayMs<u8>> Ili9341<S, SpiD, DcPin, RstPin, Delay> {
    fn change_state<NewS: State>(self) -> Ili9341<NewS, SpiD, DcPin, RstPin, Delay> {
        Ili9341::<NewS, _, _, _, _> {
            width: self.width,
            height: self.height,
            spi: self.spi,
            dc: self.dc,
            rst: self.rst,
            delay: self.delay,
            state: PhantomData,
        }
    }

    pub fn hardware_reset(mut self) -> Result<Ili9341<Disabled, SpiD, DcPin, RstPin, Delay>, Ili9341Error> {
        self.rst.set_low().map_err(|_| Ili9341Error::GpioError)?;
        self.delay.delay_ms(50);
        self.rst.set_high().map_err(|_| Ili9341Error::GpioError)?;
        self.delay.delay_ms(50);

        Ok(self.change_state::<Disabled>())
    }

    pub fn software_reset(mut self) -> Result<Ili9341<Disabled, SpiD, DcPin, RstPin, Delay>, Ili9341Error> {
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
        self.delay.delay_ms(1);

        self.dc.set_low().map_err(|_| Ili9341Error::GpioError)?;
        block!(self.spi.send(byte)).map_err(|_| Ili9341Error::SpiError)
    }

    pub fn send_data(&mut self, byte: u8) -> Result<(), Ili9341Error> {
        self.delay.delay_ms(1);

        self.dc.set_high().map_err(|_| Ili9341Error::GpioError)?;
        block!(self.spi.send(byte)).map_err(|_| Ili9341Error::SpiError)
    }

    pub fn send_packet(&mut self, command: u8, data: &[u8]) -> Result<(), Ili9341Error> {
        self.send_command(command)?;
        for byte in data {
            self.send_data(*byte)?;
        }
        Ok(())
    }
}

impl<'a, SpiD: SpiDevice, DcPin: PinId, RstPin: PinId, Delay: DelayMs<u8>> Ili9341<Disabled, SpiD, DcPin, RstPin, Delay> {
    pub fn new(
        width: u16,
        height: u16,
        spi: Spi<spi::Enabled, SpiD, 8>,
        dc: Pin<DcPin, Output<PushPull>>,
        rst: Pin<RstPin, Output<PushPull>>,
        delay: Delay,
    ) -> Ili9341<Disabled, SpiD, DcPin, RstPin, Delay> {
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

    pub fn init(self) -> Result<Ili9341<Enabled, SpiD, DcPin, RstPin, Delay>, Ili9341Error> {
        // Reconstruct as enabled
        let mut result = self.hardware_reset()?;
        result.send_init_commands()?;
        let result = result.software_reset()?;
        
        Ok(result.change_state::<Enabled>())
    }
}

impl<SpiD: SpiDevice, DcPin: PinId, RstPin: PinId, Delay: DelayMs<u8>> Ili9341<Enabled, SpiD, DcPin, RstPin, Delay> {
    /// Sets future pixel drawing operations to apply to the given bounding box. On success, returns
    /// the number of pixels (not bytes) which need to be drawn to fill this bounding box.
    /// 
    /// All bounds are inclusive - for example, to fill a 240x320 display, the bounds would be
    /// (0, 239, 0, 319).
    /// 
    /// TODO: This seems to be broken beyond drawing to the entire screen - but that's all we need,
    /// so should be fine for now
    pub fn set_pixel_drawing_area(&mut self, x1: u16, x2: u16, y1: u16, y2: u16) -> Result<u32, Ili9341Error> {
        if x2 < x1 { return Err(Ili9341Error::BoundsError) }
        if y2 < y1 { return Err(Ili9341Error::BoundsError) }

        // CASET
        self.send_packet(0x2A, &[
            ((x1 & 0xFF00) >> 8) as u8,
            (x1 & 0xFF) as u8,
            ((x2 & 0xFF00) >> 8) as u8,
            (x2 & 0xFF) as u8,
        ])?;

        // PASET
        self.send_packet(0x2B, &[
            ((y1 & 0xFF00) >> 8) as u8,
            (y1 & 0xFF) as u8,
            ((y2 & 0xFF00) >> 8) as u8,
            (y2 & 0xFF) as u8,
        ])?;

        // RAMWR
        self.send_command(0x2C)?;        

        Ok((x2 - x1 + 1) as u32 * (y2 - y1 + 1) as u32)
    }

    /// Starts a data write to the display, and returns a writer which can be used to perform
    /// writes.
    /// 
    /// This toggles the DC pin to "data", and then doesn't toggle it again for future writes, which
    /// results in a slight but noticeable speedup for large write operations compared to calling
    /// `send_data` repeatedly.
    /// 
    /// By the power of the borrow checker, the writer will prevent any methods from being called
    /// on `self` while it is alive. This stops any other methods messing up the DC pin and breaking
    /// the data stream.
    pub fn fast_data_write<'a>(&'a mut self) -> Result<Ili9341FastDataWriter<'a, SpiD, DcPin, RstPin, Delay>, Ili9341Error> {
        self.delay.delay_ms(1);

        self.dc.set_high().map_err(|_| Ili9341Error::GpioError)?;
        Ok(Ili9341FastDataWriter { ili9341: self })
    }

    /// Immediately fills the screen with the given colour.
    pub fn fill(&mut self, colour: Colour) -> Result<(), Ili9341Error> {
        // Set drawing area to cover screen
        let pixels = self.set_pixel_drawing_area(0, self.width - 1, 0, self.height - 1)?;

        // Write bytes
        let high = ((colour.0 & 0xFF00) >> 8) as u8;
        let low = (colour.0 & 0xFF) as u8;
        let mut writer = self.fast_data_write()?;
        for _ in 0..pixels {
            writer.send(high)?;
            writer.send(low)?;
        }

        Ok(())
    }

    /// Draws a sprite to fill the entire screen.
    /// 
    /// Panics if the size of the sprite does not equal the size of the screen.
    pub fn draw_screen_sprite(&mut self, sprite: &Sprite) -> Result<(), Ili9341Error> {
        if self.width != sprite.width || self.height != sprite.height {
            panic!("not a valid screen sprite");
        }

        self.set_pixel_drawing_area(0, self.width - 1, 0, self.height - 1)?;
        let mut writer = self.fast_data_write()?;
        for pixel in &sprite.data {
            writer.send((pixel.0 >> 8) as u8)?;
            writer.send((pixel.0 & 0xFF) as u8)?;
        }

        Ok(())
    }
}

impl<'a, SpiD: SpiDevice, DcPin: PinId, RstPin: PinId, Delay: DelayMs<u8>> Ili9341FastDataWriter<'a, SpiD, DcPin, RstPin, Delay> {
    pub fn send(&mut self, byte: u8) -> Result<(), Ili9341Error> {
        nb::block!(self.ili9341.spi.send(byte)).map_err(|_| Ili9341Error::SpiError)
    }
}
