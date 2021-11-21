use core::convert::TryInto;

use alloc::{vec, vec::Vec};
use rbop::{Number, UnstructuredNodeList, node::unstructured::{UnstructuredNodeRoot, Serializable}};

use crate::{interface::StorageInterface};

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
pub struct ChunkTable<'a> {
    pub start_address: u16,
    pub chunks: u16,
    pub storage: &'a mut StorageInterface,
}

impl<'a> ChunkTable<'a> {
    pub const CHUNK_SIZE: u16 = 16;
    pub const CHUNK_ADDRESS_SIZE: u16 = 2;

    fn chunk_map_address(&self) -> u16 { self.start_address }
    fn chunk_map_length(&self) -> u16 { Self::CHUNK_ADDRESS_SIZE * self.chunks }
    
    fn chunk_heap_address(&self) -> u16 { self.chunk_map_address() + self.chunk_map_length() }
    fn chunk_heap_length(&self) -> u16 { Self::CHUNK_SIZE * self.chunks }
    
    fn chunk_state_address(&self) -> u16 { self.chunk_heap_address() + self.chunk_heap_length() }
    fn chunk_state_length(&self) -> u16 { self.chunks / 8 }
    
    #[allow(dead_code)]
    fn total_length(&self) -> u16 { self.chunk_map_length() + self.chunk_heap_length() + self.chunk_state_length() }
    
    /// Given a chunk address, returns the absolute address into the storage device where this
    /// chunk's first byte is located.
    fn chunk_to_storage_address(&self, address: ChunkAddress) -> u16 {
        if address.0 > self.chunks {
            panic!("chunk {} out of range", address.0);
        }
        self.chunk_heap_address() + Self::CHUNK_SIZE * address.0
    } 
    
    /// Given a chunk index, returns the chunk address which this index maps to.
    pub fn chunk_for_index(&self, index: ChunkIndex) -> Option<ChunkAddress> {
        if index.0 >= self.chunks { return None }
        
        let chunk_address_bytes = self.storage.read(
            self.chunk_map_address() + Self::CHUNK_ADDRESS_SIZE * index.0,
            Self::CHUNK_ADDRESS_SIZE as u8
        )?;
        let chunk_address = ((chunk_address_bytes[0] as u16) << 8) | chunk_address_bytes[1] as u16;
        
        if chunk_address == 0 {
            None
        } else {
            Some(ChunkAddress(chunk_address))
        }
    }

    /// Sets a chunk index to point to a particular chunk address.
    pub fn set_chunk_for_index(&self, index: ChunkIndex, address: ChunkAddress) -> Option<()> {
        self.storage.write(
            self.chunk_map_address() + Self::CHUNK_ADDRESS_SIZE * index.0,
            &[(address.0 >> 8) as u8, (address.0 & 0xFF) as u8],
        )
    }
    
    /// Reads one chunk.
    pub fn read_chunk(&self, address: ChunkAddress) -> Option<Vec<u8>> {
        self.storage.read(self.chunk_to_storage_address(address), Self::CHUNK_SIZE as u8)
    }
    
    /// Writes one chunk.
    pub fn write_chunk(&mut self, address: ChunkAddress, data: &[u8; 16]) -> Option<()> {
        self.storage.write(self.chunk_to_storage_address(address), data)
    }

    /// Writes the given bytes onto the heap starting from the given chunk address. The bytes can
    /// span more than one chunk length.
    pub fn write_bytes(&mut self, address: ChunkAddress, data: Vec<u8>) -> Option<()> {
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
            let chunk_state_byte_address = self.chunk_state_address() + ci;
            let chunk_state_byte = self.storage.read(chunk_state_byte_address, 1)?[0];
            
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
                mask = mask >> 1;
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
    pub fn iter_bytes(&'a self, address: ChunkAddress) -> ChunkTableByteIterator<'a> {
        ChunkTableByteIterator::new(self, address)
    }
    
    /// Marks `length` chunks starting from `address` as either used or unused, regardless of the
    /// validity of this operation or their previous state.
    fn set_chunk_states(&mut self, address: ChunkAddress, length: u16, set_used: bool) -> Option<()> {
        if length == 0 { return Some(()); }
        
        // The fact that states are bit-packed makes this a bit tricker.
        // Start by building up a list of modifications to make, grouped by byte.
        let mut modifications_by_byte: Vec<(u16, u8)> = vec![];
        'outer: for i in 0..length {
            // Calculate byte to modify and bit mask to apply
            let byte_address = self.chunk_state_address() + (address.0 + i) / 8;
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
        let mut bytes = self.storage.read(modifications_by_byte[0].0, modifications_by_byte.len() as u8)?;
        for (i, (_, mask)) in modifications_by_byte.iter().enumerate() {
            if set_used {
                bytes[i] |= mask;
            } else {
                bytes[i] &= !mask;
            }
        }
        self.storage.write(modifications_by_byte[0].0, &bytes)?;
        
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
        self.storage.clear_range(self.chunk_map_address(), self.chunk_map_address())?;
        self.storage.clear_range(self.chunk_state_address(), self.chunk_state_address())?;

        if hard {
            self.storage.clear_range(self.chunk_heap_address(), self.chunk_heap_length())?;
        }

        Some(())
    }
}

pub struct ChunkTableByteIterator<'a> {
    table: &'a ChunkTable<'a>,
    buffer: [u8; ChunkTable::CHUNK_SIZE as usize],
    buffer_index: usize,
    chunk: ChunkAddress,
}

impl<'a> ChunkTableByteIterator<'a> {
    fn new(table: &'a ChunkTable<'a>, chunk: ChunkAddress) -> Self {
        let initial_buffer = table.read_chunk(chunk).unwrap().try_into().unwrap();
        Self {
            table,
            chunk,
            buffer: initial_buffer,
            buffer_index: 0,
        }
    }
}

impl<'a> Iterator for ChunkTableByteIterator<'a> {
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

pub struct Filesystem<'a> {
    pub calculations: CalculationHistory<'a>,
}

impl<'a> Filesystem<'a> {
    pub fn clear(&mut self) -> Option<()> {
        self.calculations.table.clear(false)?;

        Some(())
    }
}

pub struct CalculationHistory<'a> {
    pub table: ChunkTable<'a>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Calculation {
    pub root: UnstructuredNodeRoot,
    pub result: Option<Number>,
}

impl Calculation {
    pub fn blank() -> Self {
        Self {
            root: UnstructuredNodeRoot { root: UnstructuredNodeList { items: vec![] } },
            result: None,
        }
    }
}

impl Serializable for Calculation {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = self.root.serialize();
        if let Some(result) = self.result {
            bytes.push(1);
            bytes.append(&mut result.serialize().to_vec());
        } else {
            bytes.push(0);
        }
        bytes
    }

    fn deserialize(bytes: &mut dyn Iterator<Item = u8>) -> Option<Self> {
        let root = UnstructuredNodeRoot::deserialize(bytes)?;
        let result = match bytes.next() {
            Some(1) => {
                Number::deserialize(bytes)
            }
            Some(0) => None,
            _ => return None
        };

        Some(Calculation { root, result })
    }
}

impl<'a> CalculationHistory<'a> {
    pub fn read_calculations(&self) -> Option<Vec<Calculation>> {
        let mut idx = ChunkIndex(0);
        let mut result = vec![];
        while let Some(calc) = self.read_calculation_at_index(idx) {
            result.push(calc);
            idx.0 += 1;
        }

        Some(result)
    }

    pub fn read_calculation_at_index(&self, idx: ChunkIndex) -> Option<Calculation> {
        let chunk = self.table.chunk_for_index(idx)?;        
        Calculation::deserialize(&mut self.table.iter_bytes(chunk))
    }

    fn calculation_area_at_index(&self, idx: ChunkIndex) -> Option<(ChunkAddress, u16)> {
        // TODO: deduplicate
        let chunk = self.table.chunk_for_index(idx)?;
        let mut iterator = self.table.iter_bytes(chunk);
        Calculation::deserialize(&mut iterator)?;

        let chunks = (iterator.chunk.0 - chunk.0) + 1;

        Some((chunk, chunks))
    }

    pub fn write_calculation_at_index(
        &mut self,
        idx: ChunkIndex,
        calc: Calculation,
    ) -> Option<()> {
        // If this index was already allocated, free the heap space
        if let Some((address, length)) = self.calculation_area_at_index(idx) {
            self.table.free_chunks(address, length);
        }

        // Serialize new value
        let bytes = calc.serialize();
        
        // Allocate
        let address = self.table.allocate_chunks(self.table.chunks_required_for_bytes(bytes.len()))?;
    
        // Set index
        self.table.set_chunk_for_index(idx, address)?;

        // Write
        self.table.write_bytes(address, bytes)?;

        Some(())
    }
}
