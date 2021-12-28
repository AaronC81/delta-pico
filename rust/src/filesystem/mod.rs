pub mod chunk_table;
pub mod raw_storage;
pub mod calculation_history;
pub mod settings;
pub mod fat_interface;

pub use chunk_table::*;
pub use raw_storage::*;
pub use calculation_history::*;
pub use settings::*;
pub use fat_interface::*;

use crate::interface::framework;

pub struct Filesystem<'a> {
    pub settings: Settings<'a>,
    pub calculations: CalculationHistory<'a>,
    pub fat: FatInterface<'a>,
}

impl<'a> Filesystem<'a> {
    pub fn clear(&mut self) -> Option<()> {
        framework().storage.with_priority(|| {
            self.calculations.table.clear(false)?;
            self.fat.reset();
            // TODO: clear settings

            Some(())
        })
    }
}
