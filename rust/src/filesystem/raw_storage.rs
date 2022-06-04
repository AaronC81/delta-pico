use alloc::{vec, vec::Vec};

use crate::{interface::{StorageInterface, ApplicationFramework}, operating_system::{OperatingSystem, os_accessor, OperatingSystemPointer}};

/// A relative address into `RawStorage`.
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct RawStorageAddress(pub u16);

impl RawStorageAddress {
    pub fn offset(&self, offset: u16) -> RawStorageAddress {
        RawStorageAddress(self.0 + offset)
    }

    pub fn signed_offset(&self, offset: i16) -> RawStorageAddress {
        RawStorageAddress((self.0 as i16 + offset) as u16)
    }
}

/// A block of storage which can be used freely to store any data.
pub struct RawStorage<F: ApplicationFramework + 'static> {
    pub os: OperatingSystemPointer<F>,

    pub start_address: u16,
    pub length: u16,
}

os_accessor!(RawStorage<F>);

impl<F: ApplicationFramework> RawStorage<F> {
    /// Reads a byte from this storage area. Returns None if storage is inaccessible or
    /// out-of-bounds.
    pub fn read_byte(&mut self, address: RawStorageAddress) -> Option<u8> {
        self.read_bytes(address, 1).map(|x| x[0])
    }

    /// Reads a sequence of bytes from this storage area. Returns None if storage is inaccessible or
    /// out-of-bounds.
    pub fn read_bytes(&mut self, address: RawStorageAddress, count: u16) -> Option<Vec<u8>> {
        let mut buffer = vec![0; count as usize];
        self.os_mut().framework.storage_mut().read(self.absolute_address(address)?, &mut buffer[..])?;
        Some(buffer)
    }

    /// Writes a byte to this storage area. Returns None if storage is inaccessible or
    /// out-of-bounds.
    pub fn write_byte(&mut self, address: RawStorageAddress, byte: u8) -> Option<()> {
        self.write_bytes(address, &[byte])
    }

    /// Writes a sequence of bytes to this storage area. Returns None if storage is inaccessible or
    /// out-of-bounds.
    pub fn write_bytes(&mut self, address: RawStorageAddress, bytes: &[u8]) -> Option<()> {
        self.os_mut().framework.storage_mut().write(self.absolute_address(address)?, bytes)
    }

    /// Fills `count` bytes with the given byte value, starting from the given address.
    pub fn fill_bytes(&mut self, address: RawStorageAddress, count: u16, byte: u8) -> Option<()> {
        for offset in 0..count {
            self.write_byte(address.offset(offset), byte)?;
        }
        Some(())
    }

    /// Returns the absolute address of a `RawStorageAddress`, or None if it is invalid.
    pub fn absolute_address(&self, address: RawStorageAddress) -> Option<u16> {
        if !self.valid_address(address) {
            return None;
        }

        Some(self.start_address + address.0)
    }

    /// Returns whether a `RawStorageAddress` is within this storage area.
    pub fn valid_address(&self, address: RawStorageAddress) -> bool {
        address.0 < self.length
    }

    /// Returns the address AFTER the last storage address. This address is not valid unless
    /// subtracted from.
    pub fn end(&self) -> RawStorageAddress {
        RawStorageAddress(self.length - 1)
    }
}
