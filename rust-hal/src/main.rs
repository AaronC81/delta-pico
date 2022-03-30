//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use cortex_m::prelude::{_embedded_hal_blocking_spi_Write, _embedded_hal_spi_FullDuplex};
use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::{digital::v2::OutputPin, spi::MODE_0};
use embedded_time::{fixed_point::FixedPoint, rate::Extensions};
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
    spi::{Spi, Enabled, SpiDevice}, gpio::{FunctionSpi, Pin, PinId, Output, PushPull},
};

enum DisplayTransaction {
    Command(u8),
    Data(u8),
}

impl DisplayTransaction {
    pub fn send(&self, spi: &mut Spi<Enabled, impl SpiDevice, 8>, dc_pin: &mut Pin<impl PinId, Output<PushPull>>) {
        let byte = match self {
            Self::Command(b) => {
                dc_pin.set_low().unwrap();
                b
            }
            Self::Data(b) => {
                dc_pin.set_high().unwrap();
                b
            }
        };

        spi.write(&[*byte]).unwrap();
    }
}

#[entry]
fn main() -> ! {
    info!("Program start");
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

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.led.into_push_pull_output();
    led_pin.set_high().unwrap();

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
    delay.delay_ms(50);
    rst_pin.set_high().unwrap();
    delay.delay_ms(50);

    // DC pin
    let mut dc_pin = pins.gpio5.into_push_pull_output();

    use DisplayTransaction::*;

    let setup = [
        Command(0x0f),
        Data(0x03), Data(0x80), Data(0x02),
        Command(0xcf),
        Data(0x00), Data(0xc1), Data(0x30),
        Command(0xed),
        Data(0x64), Data(0x03), Data(0x12), Data(0x81),
        Command(0xe8),
        Data(0x85), Data(0x00), Data(0x78),
        Command(0xcb),
        Data(0x39), Data(0x2c), Data(0x00), Data(0x34), Data(0x02),
        Command(0xf7),
        Data(0x20),
        Command(0xea),
        Data(0x00), Data(0x00),
        Command(0xc0),
        Data(0x23),
        Command(0xc1),
        Data(0x10),
        Command(0xc5),
        Data(0x3e), Data(0x28),
        Command(0xc7),
        Data(0x86),
        
        Command(0x36),
        Data(0x48),
    
        Command(0x3a),
        Data(0x55),
        Command(0xb1),
        Data(0x00), Data(0x18),
        Command(0xb6),
        Data(0x08), Data(0x82), Data(0x27),
        Command(0xf2),
        Data(0x00),
        Command(0x26),
        Data(0x01),
        
        Command(0xe0),
        Data(0xf), Data(0x31), Data(0x2b), Data(0xc), Data(0xe), Data(0x8), Data(0x4e), Data(0xf1), Data(0x37), Data(0x7), Data(0x10), Data(0x3), Data(0xe), Data(0x9), Data(0x0),
    
        Command(0xe1),
        Data(0x0), Data(0xe), Data(0x14), Data(0x3), Data(0x11), Data(0x7), Data(0x31), Data(0xc1), Data(0x48), Data(0x8), Data(0xf), Data(0xc), Data(0x31), Data(0x36), Data(0xf),
    ];
    for item in setup {
        item.send(&mut spi, &mut dc_pin);
    }

    // Unsleep and display on
    Command(0x11).send(&mut spi, &mut dc_pin);
    delay.delay_ms(150);
    Command(0x29).send(&mut spi, &mut dc_pin);
    delay.delay_ms(150);

    // Clear screen
    // CASET
    Command(0x2A).send(&mut spi, &mut dc_pin);
    Data(0).send(&mut spi, &mut dc_pin); // x1 high
    Data(0).send(&mut spi, &mut dc_pin); // x1 low
    Data(0).send(&mut spi, &mut dc_pin); // x2 high
    Data(240).send(&mut spi, &mut dc_pin); // x2 low

    // PASET
    Command(0x2B).send(&mut spi, &mut dc_pin);
    Data(0).send(&mut spi, &mut dc_pin); // y1 high
    Data(0).send(&mut spi, &mut dc_pin); // y1 low
    Data(0x01).send(&mut spi, &mut dc_pin); // y2 high
    Data(0x40).send(&mut spi, &mut dc_pin); // y2 low

    // RAMWR
    Command(0x2C).send(&mut spi, &mut dc_pin);

    // Write bytes
    dc_pin.set_high().unwrap();
    for _ in 0..320 {
        for _ in 0..240 {
            nb::block!(spi.send(0)).unwrap();
            nb::block!(spi.send(0)).unwrap();
        }
    }

    loop {
        led_pin.set_low().unwrap();
        delay.delay_ms(500);
        led_pin.set_high().unwrap();
        delay.delay_ms(500);
    }
}

// End of file
