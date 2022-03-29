use alloc::{vec::Vec, string::String};

use crate::{operating_system::{os, OSInput}, interface::virtual_buttons};

macro_rules! assert_info {
    ($i:expr, $value:expr) => {
        assert_eq!(active_test_info()[$i], $value);
    };
}

pub fn run_all_tests() {
    run_calculator_tests();
}

fn run_calculator_tests() {
    // Launch calculator
    os().launch_application(
        os().application_list.applications
            .iter()
            .enumerate()
            .find(|(_, (app, _))| app.name == "Calculator")
            .unwrap()
            .0
    );

    // Clear all
    virtual_buttons::queue_virtual_button_presses(&[
        OSInput::List,
        OSInput::MoveDown,
        OSInput::Exe,
    ]);
    virtual_buttons::tick_all_virtual_buttons();
    assert_info!(1, "1"); // Includes the new blank calculation 

    // Simple calculation
    virtual_buttons::queue_virtual_button_presses(&[
        OSInput::Clear,
        OSInput::Digit(1),
        OSInput::Add,
        OSInput::Digit(3),

        OSInput::Exe,
        OSInput::MoveUp,
    ]);
    virtual_buttons::tick_all_virtual_buttons();
    assert_info!(0, "Ok(Rational(4, 1))");
    assert_info!(1, "2");

    // Fractions
    virtual_buttons::queue_virtual_button_presses(&[
        OSInput::MoveDown,
        OSInput::Digit(1),
        OSInput::Add,
        OSInput::Fraction,
        OSInput::Digit(2),
        OSInput::MoveDown,
        OSInput::Digit(3),

        OSInput::Exe,
        OSInput::MoveUp,
    ]);
    virtual_buttons::tick_all_virtual_buttons();
    assert_info!(0, "Ok(Rational(5, 3))");
    assert_info!(1, "3");

    // Decimals
    virtual_buttons::queue_virtual_button_presses(&[
        OSInput::MoveDown,
        OSInput::Digit(3),
        OSInput::Point,
        OSInput::Digit(1),
        OSInput::Digit(4),
        OSInput::Multiply,
        OSInput::Digit(2),

        OSInput::Exe,
        OSInput::MoveUp,
    ]);
    virtual_buttons::tick_all_virtual_buttons();
    assert_info!(0, "Ok(Decimal(6.28))");
    assert_info!(1, "4");

    // Negating expressions
    virtual_buttons::queue_virtual_button_presses(&[
        OSInput::MoveDown,
        OSInput::Subtract,
        OSInput::Parentheses,
        OSInput::Digit(3),
        OSInput::Add,
        OSInput::Digit(1),

        OSInput::Exe,
        OSInput::MoveUp,
    ]);
    virtual_buttons::tick_all_virtual_buttons();
    assert_info!(0, "Ok(Rational(-4, 1))");
    assert_info!(1, "5");

    // Very long expressions do not panic
    // TODO: On very large expressions, we run out of memory trying to allocate a sprite!
    virtual_buttons::queue_virtual_button_presses(&[OSInput::MoveDown]);
    for _ in 1..40 {
        virtual_buttons::queue_virtual_button_presses(&[OSInput::Digit(1)]);
    }
    virtual_buttons::queue_virtual_button_presses(&[OSInput::Exe, OSInput::MoveUp]);
    virtual_buttons::tick_all_virtual_buttons();
    assert_info!(0, "NodeError(Overflow)");
    assert_info!(1, "6");
}

fn active_test_info() -> Vec<String> {
    os().active_application.as_ref().unwrap().test_info()
}
