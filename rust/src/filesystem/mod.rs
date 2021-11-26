pub mod chunk_table;
pub mod calculation_history;

pub use chunk_table::*;
pub use calculation_history::*;

pub struct Filesystem<'a> {
    pub calculations: CalculationHistory<'a>,
}

impl<'a> Filesystem<'a> {
    pub fn clear(&mut self) -> Option<()> {
        self.calculations.table.clear(false)?;

        Some(())
    }
}
