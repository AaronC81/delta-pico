use alloc::{format, string::{String, ToString}, vec};
use rbop::{UnstructuredNodeList, nav::NavPath, node::unstructured::{UnstructuredNodeRoot, Upgradable}, render::{Area, Renderer, Viewport}};

use crate::{interface::{ButtonInput, Colour, ShapeFill}, operating_system::{OSInput, os}, rbop_impl::{RbopContext}};
use super::{Application, ApplicationInfo};
use crate::interface::framework;

pub struct MenuApplication {
    selected_index: usize,
    page_scroll_offset: usize,
}

impl MenuApplication {
    const ITEMS_PER_PAGE: usize = 5;
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
            selected_index: 0,
            page_scroll_offset: 0,
        }
    }

    fn tick(&mut self) {
        framework().display.fill_screen(Colour::BLACK);
        os().ui_draw_title("Menu");

        // Draw items
        let mut y = 40;
        for (i, (app, _)) in os().application_list.applications.iter().enumerate()
            .skip(self.page_scroll_offset).take(Self::ITEMS_PER_PAGE) {

            if i == self.selected_index {
                framework().display.draw_rect(
                    5, y, framework().display.width as i64 - 5 * 2 - 8, 54,
                    Colour::BLUE, ShapeFill::Filled, 7
                );
            }
            (framework().display.set_cursor)(65, y + 18);
            framework().display.print(app.name.clone());

            framework().display.draw_bitmap(7, y + 2, &app.icon_name());

            y += 54;
        }

        // Draw scroll amount indicator
        let scroll_indicator_column_height = 54 * Self::ITEMS_PER_PAGE;
        let scroll_indicator_bar_height_per_item = scroll_indicator_column_height / os().application_list.applications.len();
        let scroll_indicator_bar_offset = scroll_indicator_bar_height_per_item * self.page_scroll_offset;
        let scroll_indicator_bar_height = scroll_indicator_bar_height_per_item * Self::ITEMS_PER_PAGE;

        framework().display.draw_rect(
            framework().display.width as i64 - 8, 40 + scroll_indicator_bar_offset as i64,
            4, scroll_indicator_bar_height as i64, Colour::DARK_BLUE, ShapeFill::Filled, 2
        );

        (framework().display.draw)();

        if let Some(btn) = framework().buttons.wait_press() {
            match btn {
                OSInput::MoveUp => {
                    if self.selected_index == 0 {
                        // Wrap
                        self.selected_index = os().application_list.applications.len() - 1;
                        self.page_scroll_offset = os().application_list.applications.len() - Self::ITEMS_PER_PAGE;
                    } else {
                        self.selected_index -= 1;

                        // If scrolled off the screen, scroll up
                        if self.selected_index < self.page_scroll_offset {
                            self.page_scroll_offset -= 1;
                        }
                    }
                }
                OSInput::MoveDown => {
                    self.selected_index += 1;

                    // Wrap
                    if self.selected_index == os().application_list.applications.len() {
                        self.selected_index = 0;
                        self.page_scroll_offset = 0;
                    }

                    // If scrolled off the screen, scroll down
                    if self.selected_index >= self.page_scroll_offset + Self::ITEMS_PER_PAGE {
                        self.page_scroll_offset += 1;
                    }
                }
                OSInput::Exe => os().launch_application(self.selected_index),
                _ => (),
            }
        }
    }
}
