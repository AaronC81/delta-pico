use alloc::{string::{String, ToString}, vec};
use rbop::{UnstructuredNodeList, nav::NavPath, node::unstructured::{UnstructuredNodeRoot, Upgradable}, render::{Area, Renderer, Viewport}};

use crate::{interface::ButtonInput, operating_system::os, rbop_impl::{RbopContext}};
use super::{Application, ApplicationInfo};
use crate::interface::framework;

pub struct MenuApplication {
    selected_index: usize,
}

impl Application for MenuApplication {
    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "Menu".into(),
            visible: false,
        }
    }

    fn new() -> Self where Self: Sized {
        Self {
            selected_index: 0
        }
    }

    fn tick(&mut self) {
        (framework().display.fill_screen)(0);
        os().ui_draw_title("Menu");

        // Draw items
        let mut y = 40;
        for (i, (app, _)) in os().application_list.applications.iter().enumerate() {
            if i == self.selected_index {
                (framework().display.draw_rect)(
                    5, y, framework().display.width as i64 - 5 * 2, 25,
                    crate::graphics::colour::BLUE, true, 7
                );
            }
            (framework().display.set_cursor)(10, y + 4);
            framework().display.print(app.name.clone());

            y += 30;
        }

        (framework().display.draw)();

        let btn = framework().buttons.wait_press();
        match btn {
            ButtonInput::MoveUp => {
                if self.selected_index == 0 {
                    self.selected_index = os().application_list.applications.len() - 1;
                } else {
                    self.selected_index -= 1;
                }
            }
            ButtonInput::MoveDown => {
                self.selected_index += 1;
                self.selected_index %= os().application_list.applications.len();
            }
            ButtonInput::Exe => os().launch_application(self.selected_index),
            _ => (),
        }
    }
}
