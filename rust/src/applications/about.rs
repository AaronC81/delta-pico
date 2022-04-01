use alloc::format;

use crate::{interface::{Colour, ApplicationFramework}, operating_system::OperatingSystem};
use super::{Application, ApplicationInfo};

pub struct AboutApplication<'a, F: ApplicationFramework> {
    os: &'a mut OperatingSystem<'a, F>,
}

impl<'a, F: ApplicationFramework> Application for AboutApplication<'a, F> {
    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "About".into(),
            visible: true,
        }
    }

    fn new(os: &mut OperatingSystem<'a, F>) -> Self { Self { os } }

    fn tick(&mut self) {
        self.os.framework.display().fill_screen(Colour::BLACK);

        self.os.ui_draw_title("About Delta Pico");

        self.os.framework.display().print_at(5, 40,  "Software version:");
        self.os.framework.display().print_at(5, 60,  &format!("  {}", env!("CARGO_PKG_VERSION")));
        self.os.framework.display().print_at(5, 80,  &format!("  rev {}", env!("GIT_VERSION")));
        self.os.framework.display().print_at(5, 100, &format!("  rbop {}", rbop::VERSION));

        self.os.framework.display().print_at(5, 140,  "Hardware revision:");
        self.os.framework.display().print_at(5, 160,  &format!("  {}", self.os.framework.hardware_revision()));

        self.os.framework.display().print_at(70, 250,  "Created by");
        self.os.framework.display().print_at(35, 270,  "Aaron Christiansen");
        self.os.framework.display().print_at(110, 290,  ":)");

        self.os.framework.display().draw();

        // framework().buttons.wait_press();
    }
}
