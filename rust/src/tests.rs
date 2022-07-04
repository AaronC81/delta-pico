use crate::{operating_system::{OperatingSystem, OSInput, OsAccessor}, interface::{ApplicationFramework, Colour}, applications::Application};

/// Launches the suite of tests.
/// 
/// This will replace the currently running application, and as such must not be called from one!
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
    os.framework.tests_success_hook();
    os.showing_menu = true;
    os.active_application = None;
    os.ui_text_dialog("Tests passed!");
}

/// A helper method for use in application tests. Queues a sequence of virtual key presses, then
/// ticks the given `app` until the queue is empty.
/// 
/// The key presses are queued by accessing the operating system *through* the given app to avoid
/// duplicate mutable borrows when called from an application, hence the bound on `OsAccessor`.
pub fn press<F, A>(app: &mut A, inputs: &[OSInput])
where
    F: ApplicationFramework + 'static,
    A: Application<Framework = F> + OsAccessor<F>
{
    app.os_mut().queue_virtual_presses(inputs);

    while !app.os().virtual_input_queue.is_empty() {
        app.tick();
    }
}
