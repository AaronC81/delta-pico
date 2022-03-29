use alloc::vec::Vec;

use crate::interface::StorageInterface;

/// A relative address into `RawStorage`.
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct RawStorageAddress(pub u16);

/// A block of storage which can be used freely to store any data.
pub struct RawStorage<'a> {
    pub start_address: u16,
    pub length: u16,
    pub storage: &'a mut StorageInterface,
}

impl<'a> RawStorage<'a> {
    /// Reads a byte from this storage area. Returns None if storage is inaccessible or
    /// out-of-bounds.
    pub fn read_byte(&self, address: RawStorageAddress) -> Option<u8> {
        self.read_bytes(address, 1).map(|x| x[0])
    }

    /// Reads a sequence of bytes from this storage area. Returns None if storage is inaccessible or
    /// out-of-bounds.
    pub fn read_bytes(&self, address: RawStorageAddress, count: u16) -> Option<Vec<u8>> {
        self.storage.read(self.absolute_address(address)?, count)
    }

    /// Writes a byte to this storage area. Returns None if storage is inaccessible or
    /// out-of-bounds.
    pub fn write_byte(&self, address: RawStorageAddress, byte: u8) -> Option<()> {
        self.write_bytes(address, &[byte])
    }

    /// Writes a sequence of bytes to this storage area. Returns None if storage is inaccessible or
    /// out-of-bounds.
    pub fn write_bytes(&self, address: RawStorageAddress, bytes: &[u8]) -> Option<()> {
        self.storage.write(self.absolute_address(address)?, bytes)
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
