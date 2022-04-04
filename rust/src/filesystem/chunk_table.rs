use core::convert::TryInto;

use alloc::{vec, vec::Vec};

use crate::interface::{StorageInterface, ApplicationFramework};

use super::{RawStorage, RawStorageAddress};

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct ChunkAddress(pub u16);

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct ChunkIndex(pub u16);

/// A chunk table is a interface to the storage device designed for storing variable-length data
/// which can be accessed by an index.
///
/// Chunks table contain three parts:
///   - A _map_, which allows an index to be looked up, giving the chunk in the heap where that
///     index's data starts.
///   - A _heap_, all of the fixed-sized chunks in the table.
///   - A _state_, which stores a single flag for each chunk in the heap describing whether it is
///     allocated or free.
///
/// To add new data to a chunk table, first request a certain number of chunks to be allocated. This
/// will return an address of the first chunk to use, and mark the chunks as allocated so they
/// aren't allocated again. (Writing to unallocated chunks will cause issues.) You can then map an
/// index to this chunk so it can be found again later.
///
/// To modify allocated data which is a different size, free the chunks first, allocate again with
/// the new size, and update the index.
///
/// Chunk table allocations do not encode the size of the allocation; it is assumed that the size
/// can be derived by the caller from the data in the chunks.
///
/// Chunk 0 will never be allocated, so can be used as a null/unassigned value in the map.
pub struct ChunkTable<F: ApplicationFramework + 'static> {
    pub storage: RawStorage<F>,
    pub start_address: u16,
    pub chunks: u16,
}

impl<F: ApplicationFramework + 'static> ChunkTable<F> {
    pub const CHUNK_SIZE: u16 = 16;
    pub const CHUNK_ADDRESS_SIZE: u16 = 2;

    fn chunk_map_address(&self) -> RawStorageAddress { RawStorageAddress(0) }
    fn chunk_map_length(&self) -> u16 { Self::CHUNK_ADDRESS_SIZE * self.chunks }
    
    fn chunk_heap_address(&self) -> RawStorageAddress { self.chunk_map_address().offset(self.chunk_map_length()) }
    fn chunk_heap_length(&self) -> u16 { Self::CHUNK_SIZE * self.chunks }
    
    fn chunk_state_address(&self) -> RawStorageAddress { self.chunk_heap_address().offset(self.chunk_heap_length()) }
    fn chunk_state_length(&self) -> u16 { self.chunks / 8 }
    
    #[allow(dead_code)]
    fn total_length(&self) -> u16 { self.chunk_map_length() + self.chunk_heap_length() + self.chunk_state_length() }
    
    /// Given a chunk address, returns the address into the storage device where this chunk's first
    /// byte is located.
    fn chunk_to_storage_address(&self, address: ChunkAddress) -> RawStorageAddress {
        if address.0 > self.chunks {
            panic!("chunk {} out of range", address.0);
        }
        self.chunk_heap_address().offset(Self::CHUNK_SIZE * address.0)
    } 
    
    /// Given a chunk index, returns the chunk address which this index maps to.
    pub fn chunk_for_index(&mut self, index: ChunkIndex) -> Option<ChunkAddress> {
        if index.0 >= self.chunks { return None }
        
        let chunk_address_bytes = self.storage.read_bytes(
            self.chunk_map_address().offset(Self::CHUNK_ADDRESS_SIZE * index.0),
            Self::CHUNK_ADDRESS_SIZE as u16
        )?;
        let chunk_address = ((chunk_address_bytes[0] as u16) << 8) | chunk_address_bytes[1] as u16;
        
        if chunk_address == 0 {
            None
        } else {
            Some(ChunkAddress(chunk_address))
        }
    }

    /// Sets a chunk index to point to a particular chunk address.
    pub fn set_chunk_for_index(&mut self, index: ChunkIndex, address: ChunkAddress) -> Option<()>
    where [(); Self::CHUNK_SIZE as usize]: {
        self.storage.write_bytes(
            self.chunk_map_address().offset(Self::CHUNK_ADDRESS_SIZE * index.0),
            &[(address.0 >> 8) as u8, (address.0 & 0xFF) as u8],
        )
    }
    
    /// Reads one chunk.
    pub fn read_chunk(&mut self, address: ChunkAddress) -> Option<Vec<u8>> {
        self.storage.read_bytes(self.chunk_to_storage_address(address), Self::CHUNK_SIZE as u16)
    }
    
    /// Writes one chunk.
    pub fn write_chunk(&mut self, address: ChunkAddress, data: &[u8; 16]) -> Option<()> {
        self.storage.write_bytes(self.chunk_to_storage_address(address), data)
    }

    /// Writes the given bytes onto the heap starting from the given chunk address. The bytes can
    /// span more than one chunk length.
    pub fn write_bytes(&mut self, address: ChunkAddress, data: Vec<u8>) -> Option<()>
    where [(); Self::CHUNK_SIZE as usize]: {
        for (i, chunk) in data.chunks(16).enumerate() {
            let mut buffer = [0_u8; Self::CHUNK_SIZE as usize];
            for (i, b) in chunk.iter().enumerate() {
                buffer[i] = *b;
            }
            self.write_chunk(ChunkAddress(address.0 + i as u16), &buffer)?;
        }
        Some(())
    }
    
    /// Allocates `length` chunks and returns the address of the first.
    pub fn allocate_chunks(&mut self, length: u16) -> Option<ChunkAddress> {
        let mut current_free_run_start: Option<ChunkAddress> = None;
        let mut current_free_run_length: u16 = 0;

        for ci in 0..self.chunk_state_length() {
            // Grab the next 8 flags
            let chunk_state_byte_address = self.chunk_state_address().offset(ci);
            let chunk_state_byte = self.storage.read_bytes(chunk_state_byte_address, 1)?[0];
            
            // Iterate over them
            let mut mask = 0b10000000_u8;
            for cj in 0..8 {
                // Very first chunk is not allowed to be allocated, keep index pointing to 0 meaning
                // unassigned
                if !(ci == 0 && cj == 0) {
                    // Is this free?
                    if chunk_state_byte & mask == 0 {
                        // Yes, it's free; is there a run going on?
                        if let Some(start) = current_free_run_start {
                            // Increment length
                            current_free_run_length += 1;
                            
                            // Have we reached the target?
                            if current_free_run_length == length {
                                // Mark as used
                                self.set_chunk_states(start, length, true)?;
                                
                                // Return start
                                return current_free_run_start
                            }
                        } else {
                            // Start one!
                            current_free_run_start = Some(ChunkAddress(ci * 8 + cj));
                            current_free_run_length = 1;

                            // If we only needed one chunk, just mark and return now
                            if length == 1 {
                                self.set_chunk_states(current_free_run_start.unwrap(), length, true)?;
                                return current_free_run_start;
                            }
                        }
                    } else {
                        // No; reset the run
                        current_free_run_start = None;
                        current_free_run_length = 0;
                    }
                }
                
                // Move mask along
                mask >>= 1;
            }
        }
        
        // Nothing free
        None
    }
    
    /// Frees `length` chunks, starting from `address`.
    pub fn free_chunks(&mut self, address: ChunkAddress, length: u16) -> Option<()> {
        self.set_chunk_states(address, length, false)
    }

    /// Returns an iterator over the bytes in the heap, starting from the given chunk address.
    pub fn iter_bytes<'a>(&'a mut self, address: ChunkAddress) -> ChunkTableByteIterator<'a, F>
    where [(); ChunkTable::<F>::CHUNK_SIZE as usize]: {
        ChunkTableByteIterator::new(self, address)
    }
    
    /// Marks `length` chunks starting from `address` as either used or unused, regardless of the
    /// validity of this operation or their previous state.
    fn set_chunk_states(&mut self, address: ChunkAddress, length: u16, set_used: bool) -> Option<()> {
        if length == 0 { return Some(()); }
        
        // The fact that states are bit-packed makes this a bit tricker.
        // Start by building up a list of modifications to make, grouped by byte.
        let mut modifications_by_byte: Vec<(RawStorageAddress, u8)> = vec![];
        'outer: for i in 0..length {
            // Calculate byte to modify and bit mask to apply
            let byte_address = self.chunk_state_address().offset((address.0 + i) / 8);
            let bit_mask = 0b10000000 >> ((address.0 + i) % 8);
            
            // Try to find an entry for this byte in the modification list
            for (ba, mask) in modifications_by_byte.iter_mut() {
                if *ba == byte_address {
                    *mask |= bit_mask;
                    continue 'outer;
                }
            }
            
            // No entry found; add one
            modifications_by_byte.push((byte_address, bit_mask));
        }
        
        // Apply modifications
        let mut bytes = self.storage.read_bytes(modifications_by_byte[0].0, modifications_by_byte.len() as u16)?;
        for (i, (_, mask)) in modifications_by_byte.iter().enumerate() {
            if set_used {
                bytes[i] |= mask;
            } else {
                bytes[i] &= !mask;
            }
        }
        self.storage.write_bytes(modifications_by_byte[0].0, &bytes)?;
        
        Some(())
    }

    /// Returns the number of chunks required to store `bytes` bytes, assuming the bytes are written
    /// starting from the beginning of the first chunk.
    pub fn chunks_required_for_bytes(&self, bytes: usize) -> u16 {
        let mut result = bytes / Self::CHUNK_SIZE as usize;
        if bytes % Self::CHUNK_SIZE as usize > 0 {
            result += 1;
        }
        result as u16
    }

    /// Zeroes out bytes in the table, effectively clearing it.
    ///
    /// Passing `hard` as `false` will zero just the map and state bytes, and not the heap. This is
    /// all which needs to be done to get a chunk table which behaves like it's been cleared. Any
    /// previously used memory can be allocated again, though you'll have to deal with the fact that
    /// the storage chunks you get from an allocation are not guaranteed to be zero.
    ///
    /// Passing `hard` as `true` will zero the entire table, including the heap.
    pub fn clear(&mut self, hard: bool) -> Option<()> {
        self.storage.fill_bytes(self.chunk_map_address(), self.chunk_map_length(), 0)?;
        self.storage.fill_bytes(self.chunk_state_address(), self.chunk_state_length(), 0)?;

        if hard {
            self.storage.fill_bytes(self.chunk_heap_address(), self.chunk_heap_length(), 0)?;
        }

        Some(())
    }
}

pub struct ChunkTableByteIterator<'a, F: ApplicationFramework + 'static>
where [(); ChunkTable::<F>::CHUNK_SIZE as usize]: {
    pub table: &'a mut ChunkTable<F>,
    buffer: [u8; ChunkTable::<F>::CHUNK_SIZE as usize],
    buffer_index: usize,
    pub chunk: ChunkAddress,
}

impl<'a, F: ApplicationFramework + 'static> ChunkTableByteIterator<'a, F>
where [(); ChunkTable::<F>::CHUNK_SIZE as usize]: {
    fn new(table: &'a mut ChunkTable<F>, chunk: ChunkAddress) -> Self {
        let initial_buffer = table.read_chunk(chunk).unwrap().try_into().unwrap();
        Self {
            table,
            chunk,
            buffer: initial_buffer,
            buffer_index: 0,
        }
    }
}

impl<'a, F: ApplicationFramework + 'static> Iterator for ChunkTableByteIterator<'a, F>
where [(); ChunkTable::<F>::CHUNK_SIZE as usize]: {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer_index >= 16 {
            self.chunk.0 += 1;
            self.buffer = self.table.read_chunk(self.chunk)?.try_into().ok()?;
            self.buffer_index = 0;
        }

        let this_item = self.buffer[self.buffer_index];
        self.buffer_index += 1;
        Some(this_item)
    }
}
