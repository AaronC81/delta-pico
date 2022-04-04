pub mod chunk_table;
pub mod raw_storage;
// pub mod calculation_history;
pub mod settings;
// pub mod fat_interface;

pub use chunk_table::*;
pub use raw_storage::*;
// pub use calculation_history::*;
pub use settings::*;

use crate::{interface::{ApplicationFramework, StorageInterface}, operating_system::{OperatingSystem, os_accessor}};
// pub use fat_interface::*;

pub struct Filesystem<F: ApplicationFramework + 'static> {
    pub settings: Settings<F>,
    // pub calculations: CalculationHistory<'a>,
    // pub fat: FatInterface<'a>,
}

impl<F: ApplicationFramework> Filesystem<F> {
    pub fn clear(&mut self) -> Option<()> {
        // TODO
        todo!();
        Some(())
    }
}
