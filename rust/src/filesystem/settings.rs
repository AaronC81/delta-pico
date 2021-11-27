use super::{RawStorage, RawStorageAddress};

pub struct Settings<'a> {
    pub storage: RawStorage<'a>,

    pub values: SettingsValues,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SettingsValues {
    pub show_heap_usage: bool,
    pub show_frame_time: bool,
}

impl Default for SettingsValues {
    fn default() -> Self {
        SettingsValues {
            show_heap_usage: false,
            show_frame_time: false,
        }
    }
}

impl<'a> Settings<'a> {
    // These random values were chosen so that a fully 0xFF'd or 0x00'd memory can be detected, and
    // settings can be left unset.

    // The minimum size that the storage area used for this should be. Future-proofed!
    pub const MINIMUM_STORAGE_SIZE: u16 = 1028;

    /// The value of a true boolean when stored as a byte.
    const TRUE_BYTE: u8 = 0x39;

    /// The value of a false boolean when stored as a byte.
    const FALSE_BYTE: u8 = 0xB5;

    /// Returns a new `Settings` instance, loading settings from storage, or falling back to default
    /// settings if it is not accessible.
    pub fn new(storage: RawStorage<'a>) -> Self {
        let mut instance = Self {
            storage,
            values: SettingsValues::default(),
        };

        if let Some(values) = instance.load() {
            instance.values = values;
        }

        instance
    }

    /// Loads settings values from storage, using their defaults if any are not set. Returns None
    /// if storage is inaccessible.
    /// 
    /// This method is not mutating - the loaded settings values are not replaced.
    pub fn load(&self) -> Option<SettingsValues> {
        let default = SettingsValues::default();
        Some(SettingsValues {
            // TODO: index 0 should be a storage version
            show_heap_usage: self.read_bool(RawStorageAddress(1), default.show_heap_usage)?,
            show_frame_time: self.read_bool(RawStorageAddress(2), default.show_frame_time)?,
        })
    }

    /// Saves the settings values in this instance to storage. Returns None if storage is
    /// inaccessible.
    pub fn save(&mut self) -> Option<()> {
        self.write_bool(RawStorageAddress(1), self.values.show_heap_usage)?;
        self.write_bool(RawStorageAddress(2), self.values.show_frame_time)?;
        Some(())
    }

    /// Loads a boolean from storage, or falls back to a given default if no valid boolean is
    /// stored. Returns None if storage is inaccessible.
    fn read_bool(&self, address: RawStorageAddress, default: bool) -> Option<bool> {
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
