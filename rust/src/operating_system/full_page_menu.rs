use alloc::{string::String, vec::Vec};

use crate::interface::{ApplicationFramework, ShapeFill, Colour, DisplayInterface};

use super::{OperatingSystemPointer, os_accessor, OperatingSystem};

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct UIFullPageMenuItem {
    pub title: String,
    pub icon: String,
    pub toggle: Option<bool>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct UIFullPageMenu<F: ApplicationFramework + 'static> {
    os: OperatingSystemPointer<F>,
    pub items: Vec<UIFullPageMenuItem>,
    pub selected_index: usize,
    page_scroll_offset: usize,
}

impl<F: ApplicationFramework> UIFullPageMenu<F> {
    const ITEMS_PER_PAGE: usize = 5;

    pub fn new(os: OperatingSystemPointer<F>, items: Vec<UIFullPageMenuItem>) -> Self {
        Self {
            os,
            items,
            selected_index: 0,
            page_scroll_offset: 0,
        }
    }

    pub fn draw(&mut self) {
        // Draw items
        let mut y = (OperatingSystem::<F>::TITLE_BAR_HEIGHT + 10) as i16;

        // Bail early if no items
        if self.items.is_empty() {
            self.os_mut().display_sprite.print_at(75, y, "No items");
            return;
        }

        for (i, item) in self.items.iter().enumerate().skip(self.page_scroll_offset).take(Self::ITEMS_PER_PAGE) {
            // Work out whether we need to wrap
            // TODO: not an exact width
            let (lines, _, _) = self.os_mut().display_sprite.wrap_text(&item.title, 120);

            if i == self.selected_index {
                self.os_mut().display_sprite.draw_rect(
                    5, y, self.os().framework.display().width() - 5 * 2 - 8, 54,
                    Colour::BLUE, ShapeFill::Filled, 7
                );
            }

            if lines.len() == 1 {
                self.os_mut().display_sprite.print_at(65, y + 18, &lines[0]);
            } else {
                self.os_mut().display_sprite.print_at(65, y + 7, &lines[0]);
                self.os_mut().display_sprite.print_at(65, y + 28 , &lines[1]);
            }
            self.os_mut().display_sprite.draw_bitmap(7, y + 2, &item.icon);

            // Draw toggle, if necessary
            if let Some(toggle_position) = item.toggle {
                let toggle_bitmap_name = if toggle_position { "toggle_on" } else { "toggle_off" };
                self.os_mut().display_sprite.draw_bitmap(195, y + 20, toggle_bitmap_name);
            }

            y += 54;
        }

        // Draw scroll amount indicator
        let scroll_indicator_column_height = 54 * Self::ITEMS_PER_PAGE;
        let scroll_indicator_bar_height_per_item = scroll_indicator_column_height / self.items.len();
        let scroll_indicator_bar_offset = scroll_indicator_bar_height_per_item * self.page_scroll_offset;
        let scroll_indicator_bar_height = scroll_indicator_bar_height_per_item * core::cmp::min(Self::ITEMS_PER_PAGE, self.items.len());

        self.os_mut().display_sprite.draw_rect(
            self.os_mut().display_sprite.width as i16 - 8, 40 + scroll_indicator_bar_offset as i16,
            4, scroll_indicator_bar_height as u16, Colour::DARK_BLUE, ShapeFill::Filled, 2
        );
    }

    pub fn move_up(&mut self) {
        if self.selected_index == 0 {
            // Wrap
            self.selected_index = self.items.len() - 1;

            if self.items.len() > Self::ITEMS_PER_PAGE {
                self.page_scroll_offset = self.items.len() - Self::ITEMS_PER_PAGE;
            } else {
                self.page_scroll_offset = 0;
            }
        } else {
            self.selected_index -= 1;

            // If scrolled off the screen, scroll up
            if self.selected_index < self.page_scroll_offset {
                self.page_scroll_offset -= 1;
            }
        }
    }

    pub fn move_down(&mut self) {
        self.selected_index += 1;

        // Wrap
        if self.selected_index == self.items.len() {
            self.selected_index = 0;
            self.page_scroll_offset = 0;
        }

        // If scrolled off the screen, scroll down
        if self.selected_index >= self.page_scroll_offset + Self::ITEMS_PER_PAGE {
            self.page_scroll_offset += 1;
        }
    }
}

os_accessor!(UIFullPageMenu<F>);
