use crate::{interface::{ApplicationFramework, ButtonInput}, tests, operating_system::OSInput};

use super::GraphApplication;

pub fn test<F: ApplicationFramework>(app: &mut GraphApplication<F>) {
    // On launch, there should be no plots
    assert_eq!(app.plots.len(), 0);

    // Open the menu and create a new plot
    tests::press(app, &[
        // Open "Plots..."
        OSInput::Button(ButtonInput::List),
        OSInput::Button(ButtonInput::Exe),

        // Select "Add plot"
        OSInput::Button(ButtonInput::Exe),

        // Type "3x"
        OSInput::Button(ButtonInput::Digit(3)),
        OSInput::Button(ButtonInput::List),
        OSInput::Button(ButtonInput::Exe),

        // Submit
        OSInput::Button(ButtonInput::Exe),
    ]);
    assert_eq!(app.plots.len(), 1);

    // Check that value was calculated correctly into cache (when x = 3, y = 9)
    let x_to_screen_3_before = app.calculated_view_window.x_to_screen(3.into()).unwrap() as usize;
    assert_eq!(
        app.plots[0].y_values[x_to_screen_3_before].as_ref().map(|y| y.0),
        Ok(9.into()),
    );

    // Scroll over a bit and check that the index in the cache has changed, but when looked up
    // correctly, the value is still correct
    tests::press(app, &[
        OSInput::Button(ButtonInput::MoveRight),
        OSInput::Button(ButtonInput::MoveRight),
        OSInput::Button(ButtonInput::MoveRight),
        OSInput::Button(ButtonInput::MoveRight),
    ]);
    let x_to_screen_3_after = app.calculated_view_window.x_to_screen(3.into()).unwrap() as usize;
    assert_ne!(x_to_screen_3_before, x_to_screen_3_after);
    assert_eq!(
        app.plots[0].y_values[x_to_screen_3_after].as_ref().map(|y| y.0),
        Ok(9.into()),
    );

    // Scrolling down should keep the same index
    tests::press(app, &[
        OSInput::Button(ButtonInput::MoveDown),
        OSInput::Button(ButtonInput::MoveDown),
        OSInput::Button(ButtonInput::MoveDown),
        OSInput::Button(ButtonInput::MoveDown),
    ]);
    assert_eq!(
        app.plots[0].y_values[x_to_screen_3_after].as_ref().map(|y| y.0),
        Ok(9.into()),
    );

    // Remove the plot
    tests::press(app, &[
        // Open "Plots..."
        OSInput::Button(ButtonInput::List),
        OSInput::Button(ButtonInput::Exe),

        // Select the one plot
        OSInput::Button(ButtonInput::MoveDown),
        OSInput::Button(ButtonInput::Exe),

        // Delete it
        OSInput::Button(ButtonInput::MoveDown),
        OSInput::Button(ButtonInput::Exe),
    ]);
    assert_eq!(app.plots.len(), 0);
}
