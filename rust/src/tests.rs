use crate::{operating_system::OperatingSystem, interface::{ApplicationFramework, Colour}};

pub fn run_test_suite<F: ApplicationFramework + 'static>(os: &mut OperatingSystem<F>) {
    // We want to start with a blank history
    // Ideally we'd just control the app to clear this, but the app restarts after clearing,
    // which causes a lock-up (not surprising given how careful we need to be with `self` here)
    os.display_sprite.fill(Colour::BLACK);
    os.display_sprite.print_at(10, 10, "Clearing history...");
    os.draw();
    os.filesystem.calculations.table.clear(false);

    // Kick off calculator tests
    os.launch_application_by_name("Calculator");
    os.application_to_tick().test();

    // Failures are panics, so all good if we got here
    os.showing_menu = true;
    os.active_application = None;
    os.ui_text_dialog("Tests passed!");
}
