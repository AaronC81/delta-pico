use alloc::{vec, vec::Vec};
use rbop::{Number, UnstructuredNodeList, node::unstructured::{Serializable, UnstructuredNodeRoot}, error::{NodeError, MathsError}};

use crate::filesystem::chunk_table::ChunkIndex;

use super::chunk_table::{ChunkAddress, ChunkTable};

pub struct CalculationHistory<'a> {
    pub table: ChunkTable<'a>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Calculation {
    pub root: UnstructuredNodeRoot,
    pub result: CalculationResult,
}
    
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum CalculationResult {
    Ok(Number),
    NodeError(NodeError),
    MathsError(MathsError),
    None,
}

impl Calculation {
    pub fn blank() -> Self {
        Self {
            root: UnstructuredNodeRoot { root: UnstructuredNodeList { items: vec![] } },
            result: CalculationResult::None,
        }
    }
}

impl Serializable for Calculation {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = self.root.serialize();
        match &self.result {
            CalculationResult::None => bytes.push(0),
            CalculationResult::Ok(result) => {
                bytes.push(1);
                bytes.append(&mut result.serialize().to_vec());
            },
            CalculationResult::NodeError(err) => {
                bytes.push(2);
                bytes.append(&mut err.serialize().to_vec());
            },
            CalculationResult::MathsError(err) => {
                bytes.push(3);
                bytes.append(&mut err.serialize().to_vec());
            },
        }
        bytes
    }

    fn deserialize(bytes: &mut dyn Iterator<Item = u8>) -> Option<Self> {
        let root = UnstructuredNodeRoot::deserialize(bytes)?;
        let result = match bytes.next() {
            Some(0) => CalculationResult::None,
            Some(1) => CalculationResult::Ok(Number::deserialize(bytes)?),
            Some(2) => CalculationResult::NodeError(NodeError::deserialize(bytes)?),
            Some(3) => CalculationResult::MathsError(MathsError::deserialize(bytes)?),
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
