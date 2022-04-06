pub trait StorageInterface {
    #[allow(clippy::wrong_self_convention)] // &mut self required for SPI transmission
    fn is_connected(&mut self) -> bool;

    #[allow(clippy::wrong_self_convention)] // &mut self required for SPI transmission
    fn is_busy(&mut self) -> bool;

    fn write(&mut self, address: u16, bytes: &[u8]) -> Option<()>;
    fn read(&mut self, address: u16, bytes: &mut [u8]) -> Option<()>;

    fn acquire_priority(&mut self);
    fn release_priority(&mut self);

    fn with_priority<T, F>(&mut self, func: F) -> T where F : FnOnce() -> T {
        self.acquire_priority();
        let result = func();
        self.release_priority();
        result
    }

    fn clear_range(&mut self, start: u16, length: u16) -> Option<()> {
        const CHUNK_SIZE: u8 = 64;
        let buffer = [0; CHUNK_SIZE as usize];

        let mut bytes_remaining = length;
        let mut address = start;
        while bytes_remaining > 0 {
            let this_chunk_size = core::cmp::min(CHUNK_SIZE as u16, bytes_remaining);
            self.write(address, &buffer[..])?;

            address += this_chunk_size;
            bytes_remaining -= this_chunk_size;
        }

        Some(())
    }
}
