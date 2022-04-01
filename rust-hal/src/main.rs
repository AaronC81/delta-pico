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

use core::{alloc::Layout, panic::PanicInfo};

use alloc_cortex_m::CortexMHeap;
use button_matrix::ButtonEvent;
use cortex_m::{prelude::{_embedded_hal_blocking_spi_Write, _embedded_hal_spi_FullDuplex, _embedded_hal_blocking_delay_DelayMs}, delay::Delay};
use cortex_m_rt::entry;
use delta_pico_rust::{interface::{DisplayInterface, ApplicationFramework}, delta_pico_main};
use embedded_hal::{digital::v2::OutputPin, spi::MODE_0, blocking::delay::DelayMs, can::Frame};
use embedded_time::{fixed_point::FixedPoint, rate::Extensions};

use ili9341::Ili9341;
// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
    spi::{Spi, Enabled, SpiDevice}, gpio::{FunctionSpi, Pin, PinId, Output, PushPull, bank0::Gpio25, FunctionI2C}, I2C,
};
use shared_bus::BusManagerSimple;
use util::saturating_into::SaturatingInto;

use crate::graphics::{DrawingSurface, Colour, Sprite};

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

static mut LED_PIN: Option<Pin<Gpio25, Output<PushPull>>> = None;
static mut DELAY: Option<Delay> = None;

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
    ili.fill(Colour::BLACK).unwrap();

    // Create screen sprite
    let mut sprite = Sprite::new(240, 320);
    sprite.fill_surface(Colour(0xF000)).unwrap();
    sprite.draw_filled_rect(10, 10, 30, 30, Colour(0x000F)).unwrap();
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
    let shared_i2c = BusManagerSimple::new(i2c);
    let col_pcf = pcf8574::Pcf8574::new(0x38, shared_i2c.acquire_i2c());
    let row_pcf = pcf8574::Pcf8574::new(0x3E, shared_i2c.acquire_i2c());

    // Init button matrix and wait for key
    let mut buttons = button_matrix::ButtonMatrix::new(
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
            screen_sprite: sprite
        }
    };
    delta_pico_main(framework);

    loop {
        panic!("ended")
    }
}

struct DisplayImpl<SpiD: SpiDevice, DcPin: PinId, RstPin: PinId, Delay: DelayMs<u8>> {
    ili: Ili9341<ili9341::Enabled, SpiD, DcPin, RstPin, Delay>,
    screen_sprite: Sprite,
}

impl<SpiD: SpiDevice, DcPin: PinId, RstPin: PinId, Delay: DelayMs<u8>> DisplayInterface for DisplayImpl<SpiD, DcPin, RstPin, Delay> {
    type Sprite = ();

    fn width(&self) -> u64 { 240 }
    fn height(&self) -> u64 { 320 }

    fn new_sprite(&mut self, width: i16, height: i16) -> Self::Sprite { todo!() }
    fn switch_to_sprite(&mut self, sprite: &mut Self::Sprite) { todo!() }

    fn switch_to_screen(&mut self) {}

    fn fill_screen(&mut self, c: delta_pico_rust::interface::Colour) {
        self.screen_sprite.fill_surface(Colour(c.0)).unwrap();
    }

    fn draw_char(&mut self, x: i64, y: i64, character: u8) { } 
    fn draw_line(&mut self, x1: i64, y1: i64, x2: i64, y2: i64, c: delta_pico_rust::interface::Colour) { }
    fn draw_rect(&mut self, x1: i64, y1: i64, w: i64, h: i64, c: delta_pico_rust::interface::Colour, fill: delta_pico_rust::interface::ShapeFill, radius: u16) {
        self.screen_sprite.draw_filled_rect(
            x1.saturating_into(),
            y1.saturating_into(),
            w.saturating_into(),
            h.saturating_into(),
            Colour(c.0),
        ).unwrap();
    }
    fn draw_sprite(&mut self, x: i64, y: i64, sprite: &Self::Sprite) { }
    fn draw_bitmap(&mut self, x: i64, y: i64, name: &str) { }
    fn print(&mut self, s: &str) { }

    fn set_cursor(&mut self, x: i64, y: i64) { }
    fn get_cursor(&self) -> (i64, i64) { (0, 0) }

    fn set_font_size(&mut self, size: delta_pico_rust::interface::FontSize) { }
    fn get_font_size(&self) -> delta_pico_rust::interface::FontSize { delta_pico_rust::interface::FontSize::Default }

    fn draw(&mut self) {
        self.ili.draw_screen_sprite(&self.screen_sprite).unwrap();
    }
}

struct FrameworkImpl<SpiD: SpiDevice, DcPin: PinId, RstPin: PinId, Delay: DelayMs<u8>> {
    display: DisplayImpl<SpiD, DcPin, RstPin, Delay>,
}

impl<SpiD: SpiDevice, DcPin: PinId, RstPin: PinId, Delay: DelayMs<u8>> ApplicationFramework for FrameworkImpl<SpiD, DcPin, RstPin, Delay> {
    type DisplayI = DisplayImpl<SpiD, DcPin, RstPin, Delay>;

    fn display(&self) -> &Self::DisplayI { &self.display }
    fn display_mut(&mut self) -> &mut Self::DisplayI { &mut self.display }

    fn hardware_revision(&self) -> alloc::string::String {
        "Delta Pico Testing".into()
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
