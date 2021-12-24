use core::convert::TryInto;

use alloc::{vec, vec::Vec};
use fatfs::{Read, IoBase, Write, Seek};

use crate::interface::{StorageInterface, framework};

use super::{RawStorage, RawStorageAddress};

const CAT24C_PAGE_SIZE: usize = 64;

pub struct FatInterface<'a> {
    pub storage: RawStorage<'a>,
    pointer: RawStorageAddress,
}

impl<'a> FatInterface<'a> {
    pub fn new(storage: RawStorage<'a>) -> FatInterface<'a> {
        FatInterface {
            storage,
            pointer: RawStorageAddress(0),
        }
    }

    pub fn reset(&self) -> Option<()> {
        let block_num = framework().usb_mass_storage.block_num;
        let block_size = framework().usb_mass_storage.block_size;
    
        let mut fat12_fs = vec![0u8; block_num * block_size];
    
        // Add boot sector
        let boot_sector = [
            0xEB, 0x3C, 0x90, 0x4D, 0x53, 0x44, 0x4F, 0x53, 0x35, 0x2E, 0x30, 0x00, 0x02, 0x01, 0x01, 0x00,
            0x01, 0x10, 0x00, 0x10, 0x00, 0xF8, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x29, 0x34, 0x12, 0x00, 0x00, b'D' , b'e' , b'l' , b't' , b'a' ,
            b' ' , b'P' , b'i' , b'c' , b'o' , b' ' , 0x46, 0x41, 0x54, 0x31, 0x32, 0x20, 0x20, 0x20, 0x00, 0x00,
    
            // Zero up to 2 last bytes of FAT magic code
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x55, 0xAA
        ];
        fat12_fs.splice(0..(block_size - 1), boot_sector);
    
        // Add FAT12 file allocation table
        fat12_fs.splice(block_size..(block_size * 2 - 1), [0xF8, 0xFF, 0xFF, 0xFF, 0x0F]);
    
        // Add volume label and root directory
        let readme =
            "Your Delta Pico is mounted as USB flash storage. Eject the drive in your OS once done!";
        fat12_fs.splice((block_size * 2)..(block_size * 3 - 1), [
            b'D' , b'e' , b'l' , b't' , b'a' , b' ' , b'P' , b'i' , b'c' , b'o' , b' ' , 0x08, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4F, 0x6D, 0x65, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]);
    
        // Add README.txt file content
        fat12_fs.splice((block_size * 3)..(block_size * 4 - 1), readme.as_bytes().iter().copied());
    
        // Write to underlying storage in chunks of page size
        for i in 0..(fat12_fs.len() / CAT24C_PAGE_SIZE) {
            let address = i * CAT24C_PAGE_SIZE;

            self.storage.write_bytes(
                RawStorageAddress(address as u16),
                &fat12_fs[address..(address + CAT24C_PAGE_SIZE)]
            )?;
        }

        Some(())
    }

    pub fn read_all(&self) -> Option<Vec<u8>> {
        let block_num = framework().usb_mass_storage.block_num;
        let block_size = framework().usb_mass_storage.block_size;
    
        let mut fat12_fs = Vec::with_capacity(block_num * block_size);

        // Read from underlying storage in chunks of page size
        for i in 0..((block_num * block_size) / CAT24C_PAGE_SIZE) {
            let address = i * CAT24C_PAGE_SIZE;

            fat12_fs.append(
                &mut self.storage.read_bytes(RawStorageAddress(address as u16), CAT24C_PAGE_SIZE as u16)?
            );
        }

        Some(fat12_fs)
    }

    pub fn write_all(&self, data: &[u8]) -> Option<()> {
        let block_num = framework().usb_mass_storage.block_num;
        let block_size = framework().usb_mass_storage.block_size;

        // Write to underlying storage in chunks of page size
        for i in 0..(block_num * block_size / CAT24C_PAGE_SIZE) {
            let address = i * CAT24C_PAGE_SIZE;

            self.storage.write_bytes(
                RawStorageAddress(address as u16),
                &data[address..(address + CAT24C_PAGE_SIZE)]
            )?;
        }

        Some(())
    }
}

impl<'a> IoBase for FatInterface<'a> {
    type Error = ();
}

// To simplify things, these IO methods saturate to the page size of the CAT24C driver  - I'm sure
// the FAT library can recover from this

impl<'a> Read for FatInterface<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        // Retrieve bytes from driver
        let bytes = self.storage.read_bytes(self.pointer, buf.len() as u16).ok_or(())?;

        // Copy into buffer
        for i in 0..bytes.len() {
            buf[i] = bytes[i];
        }
        return Ok(buf.len())
    }
}

impl<'a> Write for FatInterface<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        // Truncate the buffer to be u8-sized
        let buf =
            if buf.len() > 64 {
                &buf[0..64]
            } else {
                &buf[0..buf.len()]
            };
        
        self.storage.write_bytes(self.pointer, buf).ok_or(())?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        // Our driver isn't clever enough to need to flush
        Ok(())
    }
}

impl<'a> Seek for FatInterface<'a> {
    fn seek(&mut self, pos: fatfs::SeekFrom) -> Result<u64, Self::Error> {
        self.pointer = match pos {
            fatfs::SeekFrom::Start(x) =>
                RawStorageAddress(x.try_into().map_err(|_| ())?),
                
            fatfs::SeekFrom::End(x) =>
                RawStorageAddress(
                    (self.storage.end().0 as i64 + x).try_into().map_err(|_| ())?
                ),
            
            fatfs::SeekFrom::Current(x) =>
                RawStorageAddress(
                    (self.pointer.0 as i64 + x).try_into().map_err(|_| ())?
                ),
        };

        Ok(self.pointer.0 as u64)
    }
}