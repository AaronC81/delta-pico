pub mod chunk_table;
pub mod raw_storage;
pub mod calculation_history;
pub mod settings;

pub use chunk_table::*;
pub use raw_storage::*;
pub use calculation_history::*;
pub use settings::*;

pub struct Filesystem<'a> {
    pub settings: Settings<'a>,
    pub calculations: CalculationHistory<'a>,
}

impl<'a> Filesystem<'a> {
    pub fn clear(&mut self) -> Option<()> {
        self.calculations.table.clear(false)?;
        
        // TODO: clear settings

        Some(())
    }
}
