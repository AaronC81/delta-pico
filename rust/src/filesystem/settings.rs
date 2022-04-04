use crate::{interface::{StorageInterface, ApplicationFramework}, operating_system::{OperatingSystem, os_accessor}};

use super::{RawStorage, RawStorageAddress};

pub struct Settings<F: ApplicationFramework + 'static> {
    pub storage: RawStorage<F>,
    pub values: SettingsValues,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SettingsValues {
    pub show_heap_usage: bool,
    pub show_frame_time: bool,
    pub fire_button_press_only: bool,
}

impl Default for SettingsValues {
    fn default() -> Self {
        SettingsValues {
            show_heap_usage: false,
            show_frame_time: false,
            fire_button_press_only: true,
        }
    }
}

impl<F: ApplicationFramework> Settings<F> {
    // These random values were chosen so that a fully 0xFF'd or 0x00'd memory can be detected, and
    // settings can be left unset.

    // The minimum size that the storage area used for this should be. Future-proofed!
    pub const MINIMUM_STORAGE_SIZE: u16 = 1028;

    /// The value of a true boolean when stored as a byte.
    const TRUE_BYTE: u8 = 0x39;

    /// The value of a false boolean when stored as a byte.
    const FALSE_BYTE: u8 = 0xB5;

    /// Returns a new `Settings` instance with default settings.
    pub fn new(storage: RawStorage<F>) -> Self {
        Self {
            storage,
            values: SettingsValues::default(),
        }
    }

    /// Loads settings values from storage, using their defaults if any are not set. Returns None
    /// if storage is inaccessible. Despite taking `mut self` due to use of the I2C bus, this does
    /// not mutate any values.
    pub fn load(&mut self) -> Option<SettingsValues> {
        let default = SettingsValues::default();
        Some(SettingsValues {
            // TODO: index 0 should be a storage version
            show_heap_usage: self.read_bool(RawStorageAddress(1), default.show_heap_usage)?,
            show_frame_time: self.read_bool(RawStorageAddress(2), default.show_frame_time)?,
            fire_button_press_only: self.read_bool(RawStorageAddress(3), default.fire_button_press_only)?,
        })
    }

    /// Loads settings values from storage and replaces the values in this instance with those
    /// loaded. If loading fails, keeps using the existing values.
    pub fn load_into_self(&mut self) {
        if let Some(values) = self.load() {
            self.values = values;
        }
    }

    /// Saves the settings values in this instance to storage. Returns None if storage is
    /// inaccessible.
    pub fn save(&mut self) -> Option<()> {
        self.write_bool(RawStorageAddress(1), self.values.show_heap_usage)?;
        self.write_bool(RawStorageAddress(2), self.values.show_frame_time)?;
        self.write_bool(RawStorageAddress(3), self.values.fire_button_press_only)?;
        Some(())
    }

    /// Loads a boolean from storage, or falls back to a given default if no valid boolean is
    /// stored. Returns None if storage is inaccessible.
    fn read_bool(&mut self, address: RawStorageAddress, default: bool) -> Option<bool> {
        let byte = self.storage.read_byte(address)?;
        match byte {
            Self::TRUE_BYTE => Some(true),
            Self::FALSE_BYTE => Some(false),
            _ => Some(default),
        }
    }

    /// Writes a boolean to storage. Returns None if storage is inaccessible.
    fn write_bool(&mut self, address: RawStorageAddress, value: bool) -> Option<()> {
        let byte = if value { Self::TRUE_BYTE } else { Self::FALSE_BYTE };
        self.storage.write_byte(address, byte)
    }
}
