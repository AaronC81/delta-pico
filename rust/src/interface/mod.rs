use alloc::string::String;

mod display;
pub use display::*;

mod buttons;
pub use buttons::*;

mod storage;
pub use storage::*;

// mod usb_mass_storage;
// pub use usb_mass_storage::*;

pub trait ApplicationFramework {
    type DisplayI : DisplayInterface;
    type ButtonsI : ButtonsInterface;
    type StorageI : StorageInterface;

    fn display(&self) -> &Self::DisplayI;
    fn display_mut(&mut self) -> &mut Self::DisplayI;

    fn buttons(&self) -> &Self::ButtonsI;
    fn buttons_mut(&mut self) -> &mut Self::ButtonsI;

    fn storage(&self) -> &Self::StorageI;
    fn storage_mut(&mut self) -> &mut Self::StorageI;

    fn hardware_revision(&self) -> String;
    fn reboot_into_bootloader(&mut self) -> !;

    fn millis(&self) -> u64;
    fn micros(&self) -> u64;

    /// Get the number of (used, total available) bytes of memory.
    fn memory_usage(&self) -> (usize, usize);

    /// Print a debug message. Currently only implemented on the simulator.
    fn debug(&self, message: &str);

    /// Called once on boot to determine whether to run the test suite.
    fn should_run_tests(&mut self) -> bool;

    /// Called immediately after a test run, started by `should_run_tests`, completes successfully.
    /// (Test failures are a panic instead.)
    fn tests_success_hook(&mut self) {}
}
