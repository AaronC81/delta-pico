//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

mod ili9341;
mod pcf8574;
mod graphics;
mod util;

use core::{alloc::Layout, panic::PanicInfo};

use alloc_cortex_m::CortexMHeap;
use cortex_m::{prelude::{_embedded_hal_blocking_spi_Write, _embedded_hal_spi_FullDuplex}, delay::Delay};
use cortex_m_rt::entry;
use embedded_hal::{digital::v2::OutputPin, spi::MODE_0};
use embedded_time::{fixed_point::FixedPoint, rate::Extensions};

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
        &mut spi,
        &mut dc_pin,
        &mut rst_pin,
        unsafe { DELAY.as_mut().unwrap() },
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
    let mut col_pcf = pcf8574::Pcf8574::new(0x38, shared_i2c.acquire_i2c());
    let mut row_pcf = pcf8574::Pcf8574::new(0x3E, shared_i2c.acquire_i2c());

    // Init button matrix
    col_pcf.write(0xFF).unwrap();
    'main: loop {
        for row in 0u8..7 {
            // Set all bits except this row
            let row_value = !(1 << row);
            row_pcf.write(row_value).unwrap();

            // Check if any buttons in this row were pressed
            let mut byte = col_pcf.read().unwrap();
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

                // Map row and column to actual numbers, rather than PCF8574 wiring
                // TODO

                // Indicate to the caller that a button was pressed
                break 'main;
            }
        }
    }

    loop {
        blink(1000);
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
