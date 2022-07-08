#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(default_alloc_error_handler)]

extern crate alloc;

mod ili9341;
mod pcf8574;
mod cat24c;
mod button_matrix;
mod rev;

use alloc::string::{String, ToString};
use alloc_cortex_m::CortexMHeap;
use button_matrix::{RawButtonEvent, ButtonMatrix};
use cat24c::Cat24C;
use cortex_m_rt::entry;
use delta_pico_rust::{interface::{DisplayInterface, ApplicationFramework, ButtonsInterface, ButtonEvent, StorageInterface}, delta_pico_main, graphics::Sprite};
use embedded_hal::{digital::v2::{OutputPin}, spi::MODE_0, blocking::delay::DelayMs, blocking::i2c::{Write, Read}};
use embedded_time::{fixed_point::FixedPoint, rate::Extensions};
use ili9341::Ili9341;
use rp_pico as bsp;
use bsp::{hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::{Sio, SioFifo, Spinlock},
    watchdog::Watchdog,
    spi::{Spi, SpiDevice}, gpio::{FunctionSpi, PinId, FunctionI2C, Pin, bank0::{Gpio20, Gpio21}}, I2C, Timer, multicore::{Stack, Multicore},
}, pac::I2C0};

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();
const HEAP_SIZE: usize = 240_000;

/// Asserts that a reference will be valid for the entire lifetime of the program, and returns a new
/// reference to the same object, but with a `'static` lifetime.
/// 
/// This is *very* unsafe when used incorrectly, since it can easily create invalid references.
/// It's also used to create multiple mutable references to the same value, which isn't allowed.
/// However, in our embedded world, we can be reasonably confident that some objects do in fact live
/// forever, even if the compiler doesn't agree with us. (Or at the very least, if these objects are
/// ever dropped, something has gone wrong enough that it won't matter any more!)
/// 
/// This avoids having to associate lifetimes with our framework implementation, which isn't
/// possible due to the `'static` bounds enforced by the OS.
fn lives_forever<T: ?Sized>(t: &mut T) -> &'static mut T {
    unsafe { (t as *mut T).as_mut().unwrap() }
}

/// The I2C bus peripheral, used to communicate with the display and storage.
/// 
/// Stored in a global so that core 1 can access it too - the HAL doesn't seem to provide a method
/// to get access to an I2C peripheral without re-initialising it, so it can't be stolen like most
/// other peripherals can.
static mut I2C: Option<&'static mut I2C<I2C0, (Pin<Gpio20, FunctionI2C>, Pin<Gpio21, FunctionI2C>)>> = None;

/// A spinlock with an arbitrarily-chosen number, used to sychronise access to the I2C bus.
type I2CSpinlock = Spinlock<8>;

/// The clock speed of the system clock in hertz. Global so that it can be read by core 1 to set up
/// delay timing.
static mut SYSTEM_CLOCK_HZ: u32 = 0;

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

    unsafe { SYSTEM_CLOCK_HZ = clocks.system_clock.freq().integer() };
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);

    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio);
    let cores = mc.cores();
    let core1 = &mut cores[1];
    let _test = core1.spawn(core1_task, unsafe { &mut CORE1_STACK.mem });

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led = pins.led.into_push_pull_output();
    led.set_high().unwrap();

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
    delay.delay_ms(50);
    rst_pin.set_high().unwrap();
    delay.delay_ms(50);

    // DC pin
    let dc_pin = pins.gpio5.into_push_pull_output();

    // Construct ILI9341 instance
    let ili = ili9341::Ili9341::new(
        240, 320,
        spi,
        dc_pin,
        rst_pin,
        lives_forever(&mut delay),
    ).init().unwrap();

    // Construct PCF8574 instances
    let sda_pin = pins.gpio20.into_mode::<FunctionI2C>();
    let scl_pin = pins.gpio21.into_mode::<FunctionI2C>();
    let mut i2c = I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        400.kHz(),
        &mut pac.RESETS,
        clocks.peripheral_clock,
    );

    unsafe { I2C = Some(lives_forever(&mut i2c)); }

    // Init flash storage
    let flash = Cat24C::new(
        0x50, 
        lives_forever(&mut i2c),
        lives_forever(&mut delay),
    );

    let framework = FrameworkImpl {
        display: DisplayImpl { ili },
        buttons: ButtonsImpl { fifo: lives_forever(&mut sio.fifo) },
        storage: StorageImpl { flash },
        
        timer,
    };
    delta_pico_main(framework);

    loop {
        panic!("ended")
    }
}

struct DisplayImpl<SpiD: SpiDevice, DcPin: PinId, RstPin: PinId, Delay: DelayMs<u8> + 'static> {
    ili: Ili9341<ili9341::Enabled, SpiD, DcPin, RstPin, Delay>,
}

impl<SpiD: SpiDevice, DcPin: PinId, RstPin: PinId, Delay: DelayMs<u8>> DisplayInterface for DisplayImpl<SpiD, DcPin, RstPin, Delay> {
    fn width(&self) -> u16 { 240 }
    fn height(&self) -> u16 { 320 }

    fn draw_display_sprite(&mut self, sprite: &Sprite) {
        self.ili.draw_screen_sprite(sprite).unwrap();
    }
}

struct ButtonsImpl {
    fifo: &'static mut SioFifo,
}

impl ButtonsInterface for ButtonsImpl {
    fn wait_event(&mut self) -> delta_pico_rust::interface::ButtonEvent {
        let raw_button = self.fifo.read_blocking();

        match RawButtonEvent::from_u32(raw_button) {
            RawButtonEvent::Press(row, col) => {
                let input = rev::BUTTON_MAPPING[row as usize][col as usize];
                return ButtonEvent::Press(input)
            }

            RawButtonEvent::Release(row, col) => {
                let input = rev::BUTTON_MAPPING[row as usize][col as usize];
                return ButtonEvent::Release(input)
            }
        };
    }

    fn poll_event(&mut self) -> Option<delta_pico_rust::interface::ButtonEvent> {
        todo!()
    }
}

struct StorageImpl<
    StorageI2CDevice: Write<Error = StorageError> + Read<Error = StorageError> + 'static,
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
    fn is_connected(&mut self) -> bool {
        let _lock = I2CSpinlock::claim();
        self.flash.is_connected()
    }
    fn is_busy(&mut self) -> bool {
        let _lock = I2CSpinlock::claim();
        self.flash.is_busy()
    }

    fn write(&mut self, address: u16, bytes: &[u8]) -> Option<()> {
        let _lock = I2CSpinlock::claim();
        self.flash.write(address, bytes).ok()
    }

    fn read(&mut self, address: u16, bytes: &mut [u8]) -> Option<()> {
        let _lock = I2CSpinlock::claim();
        self.flash.read(address, bytes).ok()
    }

    // TODO No-ops for now
    fn acquire_priority(&mut self) {}
    fn release_priority(&mut self) {}
}

struct FrameworkImpl<
    SpiD: SpiDevice,
    DcPin: PinId,
    RstPin: PinId,
    Delay: DelayMs<u8> + 'static,

    StorageI2CDevice: Write<Error = StorageError> + Read<Error = StorageError> + 'static,
    StorageError,
> {
    display: DisplayImpl<SpiD, DcPin, RstPin, Delay>,
    buttons: ButtonsImpl,
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
    type ButtonsI = ButtonsImpl;
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

    fn debug(&self, _message: &str) {
        // Not implemented
    }

    fn should_run_tests(&mut self) -> bool {
        // Hold DEL on boot
        // TODO
        // if let Ok(Some((row, col))) = self.buttons.matrix.get_raw_button() {
        //     let button = rev::BUTTON_MAPPING[row as usize][col as usize];
        //     if button == ButtonInput::Delete {
        //         return true;
        //     }
        // }
        
        false
    }
}

static mut CORE1_STACK: Stack<4096> = Stack::new();
fn core1_task() -> ! {
    let pac = unsafe { pac::Peripherals::steal() };
    let core = unsafe { pac::CorePeripherals::steal() };
    let mut sio = Sio::new(pac.SIO);
    let mut delay = cortex_m::delay::Delay::new(core.SYST, unsafe { SYSTEM_CLOCK_HZ });

    loop {
        unsafe {
            // Who needs synchronisation?
            delay.delay_ms(100);
            if I2C.is_some() { break }
        }
    }
    let i2c = unsafe { I2C.take().unwrap() };

    let col_pcf = pcf8574::Pcf8574::new(0x38, lives_forever(i2c));
    let row_pcf = pcf8574::Pcf8574::new(0x3E, lives_forever(i2c));

    // Init button matrix
    let mut buttons = ButtonMatrix::new(
        row_pcf,
        col_pcf, 
        lives_forever(&mut delay),
    );    

    loop {
        let _lock = I2CSpinlock::claim();
        if let Ok(Some(btn)) = buttons.get_event(false) {
            sio.fifo.write(btn.to_u32());
        }
        drop(_lock);

        delay.delay_ms(1);
    }
}
