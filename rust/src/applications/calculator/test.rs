use num_traits::One;
use rbop::{Number, node::structured::AngleUnit};

use crate::{interface::{ApplicationFramework, ButtonInput}, tests, operating_system::OSInput, filesystem::CalculationResult};

use super::CalculatorApplication;

pub fn test<F: ApplicationFramework>(app: &mut CalculatorApplication<F>) {
    // Note: We can assume a cleared history in here, the test setup does that for us

    // Simple calculation
    tests::press(app, &[
        OSInput::Button(ButtonInput::Digit(1)),
        OSInput::Button(ButtonInput::Add),
        OSInput::Button(ButtonInput::Digit(2)),
        OSInput::Button(ButtonInput::Exe),
    ]);
    assert!(matches!(
        app.calculations[app.calculations.len() - 2].result,
        CalculationResult::Ok(Number::Rational(3, 1))
    ));

    // Fraction
    tests::press(app, &[
        OSInput::Button(ButtonInput::Digit(1)),
        OSInput::Button(ButtonInput::Add),
        OSInput::Button(ButtonInput::Fraction),
        OSInput::Button(ButtonInput::Digit(2)),
        OSInput::Button(ButtonInput::MoveDown),
        OSInput::Button(ButtonInput::Digit(3)),
        OSInput::Button(ButtonInput::Exe),
    ]);
    assert!(matches!(
        app.calculations[app.calculations.len() - 2].result,
        CalculationResult::Ok(Number::Rational(5, 3))
    ));

    // Catalog
    app.os.filesystem.settings.values.angle_unit = AngleUnit::Degree;
    tests::press(app, &[
        // Select sine
        OSInput::Button(ButtonInput::List),
        OSInput::Button(ButtonInput::Exe),
        OSInput::Button(ButtonInput::MoveDown),
        OSInput::Button(ButtonInput::Exe),

        // Enter 90 degrees
        OSInput::Button(ButtonInput::Digit(9)),
        OSInput::Button(ButtonInput::Digit(0)),
        OSInput::Button(ButtonInput::Exe),
    ]);
    assert!(matches!(
        app.calculations[app.calculations.len() - 2].result,
        CalculationResult::Ok(Number::Decimal(d, _)) if d.is_one()
    ));
}