//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

mod ili9341;
mod pcf8574;
mod button_matrix;
mod graphics;
mod util;
mod droid_sans_20;

use core::{alloc::Layout, panic::PanicInfo};

use alloc_cortex_m::CortexMHeap;
use button_matrix::{RawButtonEvent, ButtonMatrix};
use cortex_m::{prelude::{_embedded_hal_blocking_spi_Write, _embedded_hal_spi_FullDuplex, _embedded_hal_blocking_delay_DelayMs}, delay::Delay};
use cortex_m_rt::entry;
use delta_pico_rust::{interface::{DisplayInterface, ApplicationFramework, Colour, ButtonsInterface, ButtonEvent, ButtonInput}, delta_pico_main};
use droid_sans_20::droid_sans_20_lookup;
use embedded_hal::{digital::v2::OutputPin, spi::MODE_0, blocking::delay::DelayMs, can::Frame, blocking::i2c::{Write, Read}};
use embedded_time::{fixed_point::FixedPoint, rate::Extensions};

use ili9341::Ili9341;
// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::{hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
    spi::{Spi, Enabled, SpiDevice}, gpio::{FunctionSpi, Pin, PinId, Output, PushPull, bank0::{Gpio25, Gpio20, Gpio21}, FunctionI2C}, I2C, i2c::Controller,
}, pac::I2C0};
use shared_bus::{BusManagerSimple, I2cProxy, NullMutex, BusManager};
use util::saturating_into::SaturatingInto;

use crate::graphics::{DrawingSurface, RawColour, Sprite};

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

static mut LED_PIN: Option<Pin<Gpio25, Output<PushPull>>> = None;
static mut DELAY: Option<Delay> = None;
static mut SHARED_I2C: Option<BusManager<NullMutex<I2C<I2C0, (Pin<Gpio20, FunctionI2C>, Pin<Gpio21, FunctionI2C>), Controller>>>> = None;

#[entry]
fn main() -> ! {
    // Set up allocator
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 200_000;
        static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe {
            ALLOCATOR.init((&mut HEAP).as_ptr() as usize, HEAP_SIZE)
        }
    }

    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    unsafe {
        DELAY = Some(cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer()));
    }

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    unsafe {
        LED_PIN = Some(pins.led.into_push_pull_output());
        LED_PIN.as_mut().unwrap().set_high().unwrap();
    }

    // Chip-select display
    let mut cs_pin = pins.gpio4.into_push_pull_output();
    cs_pin.set_low().unwrap();

    // Set up SPI and pins
    let spi = Spi::<_, _, 8>::new(pac.SPI0);
    let mut spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        70_000_000.Hz(),
        &MODE_0,
    );
    let _miso = pins.gpio0.into_mode::<FunctionSpi>();
    let _mosi = pins.gpio3.into_mode::<FunctionSpi>();
    let _sclk = pins.gpio2.into_mode::<FunctionSpi>();

    // Hardware reset
    let mut rst_pin = pins.gpio6.into_push_pull_output();
    rst_pin.set_low().unwrap();
    unsafe { DELAY.as_mut().unwrap() }.delay_ms(50);
    rst_pin.set_high().unwrap();
    unsafe { DELAY.as_mut().unwrap() }.delay_ms(50);

    // DC pin
    let mut dc_pin = pins.gpio5.into_push_pull_output();

    // Construct ILI9341 instance
    let mut ili = ili9341::Ili9341::new(
        240, 320,
        spi,
        dc_pin,
        rst_pin,
        unsafe { core::ptr::read(DELAY.as_ref().unwrap()) },
    ).init().unwrap();
    ili.fill(RawColour::BLACK).unwrap();

    // Create screen sprite
    let mut sprite = Sprite::new(240, 320);
    sprite.fill_surface(RawColour(0xF000)).unwrap();
    sprite.draw_filled_rect(10, 10, 30, 30, RawColour(0x000F)).unwrap();
    ili.draw_screen_sprite(&sprite).unwrap();

    // Construct PCF8574 instances
    let mut sda_pin = pins.gpio20.into_mode::<FunctionI2C>();
    let mut scl_pin = pins.gpio21.into_mode::<FunctionI2C>();
    let mut i2c = I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        400.kHz(),
        &mut pac.RESETS,
        clocks.peripheral_clock,
    );

    unsafe { SHARED_I2C = Some(BusManagerSimple::new(i2c)); }

    let col_pcf = pcf8574::Pcf8574::new(0x38, unsafe { SHARED_I2C.as_mut().unwrap().acquire_i2c() });
    let row_pcf = pcf8574::Pcf8574::new(0x3E, unsafe { SHARED_I2C.as_mut().unwrap().acquire_i2c() });

    // Init button matrix and wait for key
    let mut buttons = ButtonMatrix::new(
        row_pcf,
        col_pcf, 
        // TODO: This is not "clever" or "I know better" usage of `unsafe`, this is literally just
        // plain UB - both this and `ili` hold a mutable reference to `delay` simultaneously
        // But, like... how are you supposed to share it!?
        unsafe { DELAY.as_mut().unwrap() }
    );

    let framework = FrameworkImpl {
        display: DisplayImpl {
            ili,
            screen_sprite: sprite,

            cursor_x: 0,
            cursor_y: 0,
        },

        buttons: ButtonsImpl { matrix: buttons }
    };
    delta_pico_main(framework);

    loop {
        panic!("ended")
    }
}

struct DisplayImpl<SpiD: SpiDevice, DcPin: PinId, RstPin: PinId, Delay: DelayMs<u8>> {
    ili: Ili9341<ili9341::Enabled, SpiD, DcPin, RstPin, Delay>,
    screen_sprite: Sprite,

    cursor_x: i16,
    cursor_y: i16,
}

impl<SpiD: SpiDevice, DcPin: PinId, RstPin: PinId, Delay: DelayMs<u8>> DisplayInterface for DisplayImpl<SpiD, DcPin, RstPin, Delay> {
    type Sprite = ();

    fn width(&self) -> u16 { 240 }
    fn height(&self) -> u16 { 320 }

    fn new_sprite(&mut self, width: u16, height: u16) -> Self::Sprite { todo!() }
    fn switch_to_sprite(&mut self, sprite: &mut Self::Sprite) { todo!() }

    fn switch_to_screen(&mut self) {}

    fn fill_screen(&mut self, c: Colour) {
        self.screen_sprite.fill_surface(c.into()).unwrap();
    }

    fn draw_char(&mut self, character: u8) {
        let (x, y) = self.get_cursor();

        let character_bitmap = droid_sans_20_lookup(character);
        if character_bitmap.is_none() { return; }
        let character_bitmap = character_bitmap.unwrap();

        // TODO: anti-aliasing or any transparency
        // TODO: font colour
        // TODO: \n

        // Each character is 4bpp;, so we maintain a flip-flopping boolean of whether to read the
        // upper or lower byte
        let mut lower_byte = false;
        let mut index = 2usize;

        let width = character_bitmap[0];
        let height = character_bitmap[1];

        for ox in 0..width {
            for oy in 0..height {
                let alpha_nibble = if lower_byte {
                    lower_byte = false;
                    let x = character_bitmap[index] & 0xF;
                    index += 1;
                    x
                } else {
                    lower_byte = true;
                    (character_bitmap[index] & 0xF0) >> 4
                };

                if alpha_nibble > 0x8 {
                    if let Some(px) = self.screen_sprite.try_pixel(x + ox as i16, y + oy as i16) {
                        *px = RawColour(0xFFFF);
                    }
                }
            }
        }

        self.cursor_x += Into::<i16>::into(character_bitmap[0]) - 1;
    } 
    fn draw_line(&mut self, x1: i16, y1: i16, x2: i16, y2: i16, c: Colour) { }
    fn draw_rect(&mut self, x1: i16, y1: i16, w: u16, h: u16, c: Colour, fill: delta_pico_rust::interface::ShapeFill, radius: u16) {
        self.screen_sprite.draw_filled_rect(x1, y1, w, h, c.into()).unwrap();
    }
    fn draw_sprite(&mut self, x: i16, y: i16, sprite: &Self::Sprite) { }
    fn draw_bitmap(&mut self, x: i16, y: i16, name: &str) { }
    fn print(&mut self, s: &str) {
        for c in s.as_bytes() {
            self.draw_char(*c);
        }
    }

    fn set_cursor(&mut self, x: i16, y: i16) {
        self.cursor_x = x;
        self.cursor_y = y;
    }
    fn get_cursor(&self) -> (i16, i16) {
        (self.cursor_x, self.cursor_y)
    }

    fn set_font_size(&mut self, size: delta_pico_rust::interface::FontSize) { }
    fn get_font_size(&self) -> delta_pico_rust::interface::FontSize { delta_pico_rust::interface::FontSize::Default }

    fn draw(&mut self) {
        self.ili.draw_screen_sprite(&self.screen_sprite).unwrap();
    }
}

struct ButtonsImpl<
    RowI2CDevice: Write<Error = RowError> + Read<Error = RowError>,
    RowError,
    ColI2CDevice: Write<Error = ColError> + Read<Error = ColError>,
    ColError,
    Delay: DelayMs<u8> + 'static,
> {
    matrix: ButtonMatrix<RowI2CDevice, RowError, ColI2CDevice, ColError, Delay>,
}

impl<
    RowI2CDevice: Write<Error = RowError> + Read<Error = RowError>,
    RowError,
    ColI2CDevice: Write<Error = ColError> + Read<Error = ColError>,
    ColError,
    Delay: DelayMs<u8> + 'static,
> ButtonsInterface for ButtonsImpl<RowI2CDevice, RowError, ColI2CDevice, ColError, Delay> {
    fn wait_event(&mut self) -> delta_pico_rust::interface::ButtonEvent {
        self.matrix.get_event(true).unwrap();
        ButtonEvent::Press(ButtonInput::Exe)
    }

    fn poll_event(&mut self) -> Option<delta_pico_rust::interface::ButtonEvent> {
        todo!()
    }
}

struct FrameworkImpl<
    SpiD: SpiDevice,
    DcPin: PinId,
    RstPin: PinId,
    Delay: DelayMs<u8> + 'static,

    RowI2CDevice: Write<Error = RowError> + Read<Error = RowError>,
    RowError,
    ColI2CDevice: Write<Error = ColError> + Read<Error = ColError>,
    ColError,
> {
    display: DisplayImpl<SpiD, DcPin, RstPin, Delay>,
    buttons: ButtonsImpl<RowI2CDevice, RowError, ColI2CDevice, ColError, Delay>,
}

impl<
    SpiD: SpiDevice,
    DcPin: PinId,
    RstPin: PinId,
    Delay: DelayMs<u8> + 'static,

    RowI2CDevice: Write<Error = RowError> + Read<Error = RowError>,
    RowError,
    ColI2CDevice: Write<Error = ColError> + Read<Error = ColError>,
    ColError,
> ApplicationFramework for FrameworkImpl<SpiD, DcPin, RstPin, Delay, RowI2CDevice, RowError, ColI2CDevice, ColError> {
    type DisplayI = DisplayImpl<SpiD, DcPin, RstPin, Delay>;
    type ButtonsI = ButtonsImpl<RowI2CDevice, RowError, ColI2CDevice, ColError, Delay>;

    fn display(&self) -> &Self::DisplayI { &self.display }
    fn display_mut(&mut self) -> &mut Self::DisplayI { &mut self.display }

    fn buttons(&self) -> &Self::ButtonsI { &self.buttons }
    fn buttons_mut(&mut self) -> &mut Self::ButtonsI { &mut self.buttons }

    fn hardware_revision(&self) -> alloc::string::String {
        "Rev. 3 (Rust Framework)".into()
    }
}

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    loop {
        blink(100);
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        blink(100);
        blink(100);
        blink(100);
        blink(500);
        blink(500);
        blink(500);
        blink(100);
        blink(100);
        blink(100);

        unsafe { DELAY.as_mut().unwrap() }.delay_ms(300);        
    }
}

fn blink(time: u32) {
    unsafe { LED_PIN.as_mut().unwrap() }.set_high().unwrap();
    unsafe { DELAY.as_mut().unwrap() }.delay_ms(time);
    unsafe { LED_PIN.as_mut().unwrap() }.set_low().unwrap();
    unsafe { DELAY.as_mut().unwrap() }.delay_ms(time);
}
