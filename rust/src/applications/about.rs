use alloc::format;

use crate::{interface::{Colour, ApplicationFramework, DisplayInterface}, operating_system::{OperatingSystem, os_accessor}};
use super::{Application, ApplicationInfo};

pub struct AboutApplication<F: ApplicationFramework + 'static> {
    os: *mut OperatingSystem<F>,
}

os_accessor!(AboutApplication<F>);

impl<F: ApplicationFramework> Application for AboutApplication<F> {
    type Framework = F;

    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "About".into(),
            visible: true,
        }
    }

    fn new(os: *mut OperatingSystem<F>) -> Self { Self { os } }

    fn tick(&mut self) {
        self.os_mut().display_sprite.fill(Colour::BLACK);

        self.os_mut().ui_draw_title("About Delta Pico");
        let hw_revision = self.os().framework.hardware_revision();
        let disp = &mut self.os_mut().display_sprite;

        disp.print_at(5, 40,  "Software version:");
        disp.print_at(5, 60,  &format!("  {}", env!("CARGO_PKG_VERSION")));
        // disp.print_at(5, 80,  &format!("  rev {}", env!("GIT_VERSION")));
        disp.print_at(5, 100, &format!("  rbop {}", rbop::VERSION));

        disp.print_at(5, 140,  "Hardware revision:");
        disp.print_at(5, 160,  &format!("  {}", hw_revision));

        disp.print_at(70, 250,  "Created by");
        disp.print_at(35, 270,  "Aaron Christiansen");
        disp.print_at(110, 290,  ":)");

        self.os_mut().draw();
        self.os_mut().input();
    }
}

