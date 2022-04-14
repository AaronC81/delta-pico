//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

mod ili9341;
mod pcf8574;
mod cat24c;
mod button_matrix;
mod rev;

use core::alloc::Layout;

use alloc::string::{String, ToString};
use alloc_cortex_m::CortexMHeap;
use button_matrix::{RawButtonEvent, ButtonMatrix};
use cat24c::Cat24C;
use cortex_m::delay::Delay;
use cortex_m_rt::entry;
use delta_pico_rust::{interface::{DisplayInterface, ApplicationFramework, ButtonsInterface, ButtonEvent, StorageInterface, ButtonInput}, delta_pico_main, graphics::Sprite, operating_system::OSInput};
use embedded_hal::{digital::v2::{OutputPin, ToggleableOutputPin}, spi::MODE_0, blocking::delay::DelayMs, blocking::i2c::{Write, Read}};
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
    spi::{Spi, SpiDevice}, gpio::{FunctionSpi, Pin, PinId, Output, PushPull, bank0::{Gpio25, Gpio20, Gpio21}, FunctionI2C}, I2C, i2c::Controller, Timer, multicore::{Stack, Multicore},
}, pac::I2C0};
use shared_bus::{BusManagerSimple, NullMutex, BusManager};

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();
const HEAP_SIZE: usize = 240_000;

static mut LED_PIN: Option<Pin<Gpio25, Output<PushPull>>> = None;
static mut DELAY: Option<Delay> = None;

// I've just picked random ones. Cross your fingers!
pub type I2CSpinlock = bsp::hal::sio::Spinlock16;
pub type ButtonQueueSpinlock = bsp::hal::sio::Spinlock17;

type DeltaPicoI2C = I2C<I2C0, (Pin<Gpio20, FunctionI2C>, Pin<Gpio21, FunctionI2C>), Controller>;
static mut SHARED_I2C: Option<BusManager<NullMutex<DeltaPicoI2C>>> = None;

static mut BUTTON_QUEUE: [Option<ButtonInput>; 16] = [None; 16];

#[entry]
fn main() -> ! {
    // Set up allocator
    {
        use core::mem::MaybeUninit;
        static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe {
            ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE)
        }
    }

    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let mut sio = Sio::new(pac.SIO);

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

    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio);
    let cores = mc.cores();
    let core1 = &mut cores[1];
    let _test = core1.spawn(core1_task, unsafe { &mut CORE1_STACK.mem });

    unsafe {
        DELAY = Some(cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer()));
    }

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);

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
    let spi = spi.init(
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
    let dc_pin = pins.gpio5.into_push_pull_output();

    // Construct ILI9341 instance
    let ili = ili9341::Ili9341::new(
        240, 320,
        spi,
        dc_pin,
        rst_pin,
        unsafe { core::ptr::read(DELAY.as_ref().unwrap()) },
    ).init().unwrap();

    // Construct PCF8574 instances
    let sda_pin = pins.gpio20.into_mode::<FunctionI2C>();
    let scl_pin = pins.gpio21.into_mode::<FunctionI2C>();
    let i2c = I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        400.kHz(),
        &mut pac.RESETS,
        clocks.peripheral_clock,
    );

    unsafe { SHARED_I2C = Some(BusManagerSimple::new(i2c)); }

    // Init flash storage
    let flash = Cat24C::new(
        0x50, 
        unsafe { SHARED_I2C.as_mut().unwrap().acquire_i2c() },
        unsafe { DELAY.as_mut().unwrap() },
    );

    let framework = FrameworkImpl {
        display: DisplayImpl { ili },
        buttons: ButtonsImpl { delay: unsafe { DELAY.as_mut().unwrap() } },
        storage: StorageImpl { flash },
        
        timer,
    };
    delta_pico_main(framework);

    loop {
        panic!("ended")
    }
}

struct DisplayImpl<SpiD: SpiDevice, DcPin: PinId, RstPin: PinId, Delay: DelayMs<u8>> {
    ili: Ili9341<ili9341::Enabled, SpiD, DcPin, RstPin, Delay>,
}

impl<SpiD: SpiDevice, DcPin: PinId, RstPin: PinId, Delay: DelayMs<u8>> DisplayInterface for DisplayImpl<SpiD, DcPin, RstPin, Delay> {
    fn width(&self) -> u16 { 240 }
    fn height(&self) -> u16 { 320 }

    fn draw_display_sprite(&mut self, sprite: &Sprite) {
        self.ili.draw_screen_sprite(sprite).unwrap();
    }
}

struct ButtonsImpl<Delay: DelayMs<u8> + 'static> {
    delay: &'static mut Delay,
}

impl<Delay: DelayMs<u8> + 'static> ButtonsInterface for ButtonsImpl<Delay> {
    fn wait_event(&mut self) -> delta_pico_rust::interface::ButtonEvent {
        unsafe {
            loop {
                {
                    // If there are items in the queue, grab the first and move the rest along
                    let _lock = ButtonQueueSpinlock::claim();
                    if let Some(button) = BUTTON_QUEUE[0].take() {
                        let event = ButtonEvent::Press(button);
                        for i in 0..(BUTTON_QUEUE.len() - 1) {
                            BUTTON_QUEUE[i] = BUTTON_QUEUE[i + 1];
                        }
                        return event;
                    }
                }

                // Prevents us from holding the queue lock all the time
                self.delay.delay_ms(10);
            }
        }
    }

    fn poll_event(&mut self) -> Option<delta_pico_rust::interface::ButtonEvent> {
        todo!()
    }
}

struct StorageImpl<
    StorageI2CDevice: Write<Error = StorageError> + Read<Error = StorageError>,
    StorageError,
    Delay: DelayMs<u8> + 'static,
> {
    flash: Cat24C<StorageI2CDevice, StorageError, Delay>,
}

impl<
    StorageI2CDevice: Write<Error = StorageError> + Read<Error = StorageError>,
    StorageError,
    Delay: DelayMs<u8> + 'static,
> StorageInterface for StorageImpl<StorageI2CDevice, StorageError, Delay> {
    fn is_connected(&mut self) -> bool { self.flash.is_connected() }
    fn is_busy(&mut self) -> bool { self.flash.is_busy() }

    fn write(&mut self, address: u16, bytes: &[u8]) -> Option<()> {
        self.flash.write(address, bytes).ok()
    }

    fn read(&mut self, address: u16, bytes: &mut [u8]) -> Option<()> {
        self.flash.read(address, bytes).ok()
    }

    // No-ops for now - no multicore
    fn acquire_priority(&mut self) {}
    fn release_priority(&mut self) {}
}

struct FrameworkImpl<
    SpiD: SpiDevice,
    DcPin: PinId,
    RstPin: PinId,
    Delay: DelayMs<u8> + 'static,

    StorageI2CDevice: Write<Error = StorageError> + Read<Error = StorageError>,
    StorageError,
> {
    display: DisplayImpl<SpiD, DcPin, RstPin, Delay>,
    buttons: ButtonsImpl<Delay>,
    storage: StorageImpl<StorageI2CDevice, StorageError, Delay>,
    timer: Timer,
}

impl<
    SpiD: SpiDevice,
    DcPin: PinId,
    RstPin: PinId,
    Delay: DelayMs<u8> + 'static,

    StorageI2CDevice: Write<Error = StorageError> + Read<Error = StorageError>,
    StorageError,
> ApplicationFramework for FrameworkImpl<SpiD, DcPin, RstPin, Delay, StorageI2CDevice, StorageError> {
    type DisplayI = DisplayImpl<SpiD, DcPin, RstPin, Delay>;
    type ButtonsI = ButtonsImpl<Delay>;
    type StorageI = StorageImpl<StorageI2CDevice, StorageError, Delay>;

    fn display(&self) -> &Self::DisplayI { &self.display }
    fn display_mut(&mut self) -> &mut Self::DisplayI { &mut self.display }

    fn buttons(&self) -> &Self::ButtonsI { &self.buttons }
    fn buttons_mut(&mut self) -> &mut Self::ButtonsI { &mut self.buttons }

    fn storage(&self) -> &Self::StorageI { &self.storage }
    fn storage_mut(&mut self) -> &mut Self::StorageI { &mut self.storage }

    fn hardware_revision(&self) -> String { rev::REVISION_NAME.to_string() }

    fn reboot_into_bootloader(&mut self) -> ! {
        // Awww, yeah!
        // This is a translation of the parts of...
        //   - https://github.com/raspberrypi/pico-sdk/blob/master/src/rp2_common/pico_bootrom/bootrom.c
        //   - https://github.com/raspberrypi/pico-sdk/blob/master/src/rp2_common/pico_bootrom/include/pico/bootrom.h
        // ...required to call `reset_usb_boot`.
        // Nothing super fancy is going on here, just lots of casting pointers around.
        // The mem::transmute calls are required because Rust doesn't allow you to cast `*const _`
        // to `extern "C" fn(...) -> _`, even though the latter is still just a pointer in memory.
        unsafe {
            // Resolve a function which allows us to look up items in ROM tables
            let rom_table_lookup_fn_addr = *(0x18 as *const u16) as *const ();
            let rom_table_lookup_fn: extern "C" fn(*const u16, u32) -> *const () = core::mem::transmute(rom_table_lookup_fn_addr);
            
            // Use that function to look up the address of the USB bootloader function
            let usb_boot_fn_code = (('B' as u32) << 8) | ('U' as u32);
            let func_table = *(0x14 as *const u16) as *const u16;
            let usb_boot_fn_addr = rom_table_lookup_fn(func_table, usb_boot_fn_code);

            // Call that function
            let usb_boot_fn: extern "C" fn(u32, u32) = core::mem::transmute(usb_boot_fn_addr);
            usb_boot_fn(0, 0);
        }
        panic!("failed to access bootloader")
    }

    fn micros(&self) -> u64 {
        self.timer.get_counter()
    }

    fn millis(&self) -> u64 {
        self.micros() / 1000
    }

    fn memory_usage(&self) -> (usize, usize) {
        (ALLOCATOR.used(), HEAP_SIZE)
    }
}

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    loop {
        blink(100);
    }
}

fn blink(time: u32) {
    unsafe { LED_PIN.as_mut().unwrap() }.set_high().unwrap();
    unsafe { DELAY.as_mut().unwrap() }.delay_ms(time);
    unsafe { LED_PIN.as_mut().unwrap() }.set_low().unwrap();
    unsafe { DELAY.as_mut().unwrap() }.delay_ms(time);
}

static mut CORE1_STACK: Stack<4096> = Stack::new();
fn core1_task() -> ! {
    let mut pac = unsafe { pac::Peripherals::steal() };
    let core = unsafe { pac::CorePeripherals::steal() };

    let sio = Sio::new(pac.SIO);
    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.led.into_push_pull_output();

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

    loop {
        for _ in 0..12500000 { cortex_m::asm::nop(); }
        let _lock = ButtonQueueSpinlock::claim();
        unsafe {
            let input = match buttons.get_event(false).unwrap() {
                Some(RawButtonEvent::Press(row, col)) => {
                    let input = rev::BUTTON_MAPPING[row as usize][col as usize];
                    Some(input)
                }

                _ => None,
            };

            // Insert button press into the first free slot in the queue, if there is one
            if let Some(input) = input {
                if let Some(slot) = BUTTON_QUEUE.iter_mut().find(|b| b.is_none()) {
                    *slot = Some(input);
                }
            }
        }
        led_pin.toggle().unwrap();
    }
}
