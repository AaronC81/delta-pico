use crate::{interface::{ApplicationFramework, ButtonInput}, tests, operating_system::OSInput};

use super::GraphApplication;

pub fn test<F: ApplicationFramework>(app: &mut GraphApplication<F>) {
    // On launch, there should be no plots
    assert_eq!(app.plots.len(), 0);
}
