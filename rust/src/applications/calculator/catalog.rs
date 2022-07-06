use core::fmt::Debug;
use alloc::{string::String, vec::Vec};

use crate::{operating_system::{OperatingSystemPointer, OperatingSystem, OSInput, SelectorMenu, SelectorMenuTickResult}, interface::{ApplicationFramework, Colour, ShapeFill, ButtonInput}};

/// A pop-up dialog box with multiple columns of selectable items.
#[derive(Debug)]
pub struct Catalog<F, T>
where
    F: ApplicationFramework + 'static,
    T: Debug
{
    os: OperatingSystemPointer<F>,
    title: String,
    items: Vec<CatalogItem<T>>,
    selected_index: usize,
}

/// An item in a `Catalog`. Additional metadata can be attached to an item.
#[derive(Debug)]
pub struct CatalogItem<T>
where T: Debug
{
    pub name: String,
    pub description: String,
    pub metadata: T,
}

impl<T> CatalogItem<T>
where T: Debug
{
    pub fn new(name: impl Into<String>, description: impl Into<String>, metadata: T) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            metadata,
        }
    }
}

impl<F, T> Catalog<F, T>
where
    F: ApplicationFramework + 'static,
    T: Debug
{
    const WIDTH: u16 = 200;
    const HEIGHT: u16 = 250;
    const ROW_HEIGHT: u16 = 30;
    const ITEM_PADDING: u16 = 5;
    const DESCRIPTION_HEIGHT: u16 = 70;
    const COLUMNS: u16 = 3;

    pub fn new(os: OperatingSystemPointer<F>, title: impl Into<String>, items: Vec<CatalogItem<T>>) -> Self {
        Self {
            os,
            title: title.into(),
            items,
            selected_index: 0,
        }
    }
}

impl<F, T> SelectorMenu for Catalog<F, T>
where
    F: ApplicationFramework + 'static,
    T: Debug
{
    type Item = CatalogItem<T>;

    fn selected_index(&self) -> usize { self.selected_index }
    fn items(&self) -> &Vec<Self::Item> { &self.items }
    fn into_items(self) -> Vec<Self::Item> { self.items }

    /// Draws the catalog to the screen, takes one key of input, and indicates whether any external
    /// action needs to occur.
    fn tick(&mut self) -> SelectorMenuTickResult {
        // Calculate the X and Y position to start drawing from
        let starting_x = ((self.os.display_sprite.width - Self::WIDTH) / 2) as i16;
        let starting_y = 
            (OperatingSystem::<F>::TITLE_BAR_HEIGHT +
            ((self.os.display_sprite.height - OperatingSystem::<F>::TITLE_BAR_HEIGHT) - Self::HEIGHT) / 2) as i16;
        
        // Draw background and title
        // We don't clear the screen, so we don't disrupt the app behind
        // (This is fine since the size of this dialog never changes)
        self.os.display_sprite.draw_rect(
            starting_x, starting_y, Self::WIDTH, Self::HEIGHT,
            Colour::GREY, ShapeFill::Filled, 0,
        );
        self.os.ui_draw_title(&self.title);

        // Draw items
        let mut column = 0;
        let mut row = 0;
        for (i, item) in self.items.iter().enumerate() {
            let item_x = starting_x + (column * (Self::WIDTH / Self::COLUMNS) - if column > 0 { 1 } else { 0 }) as i16;
            let item_y = starting_y + (row * (Self::ROW_HEIGHT - 1)) as i16;

            // Highlight if selected
            if self.selected_index == i {
                self.os.display_sprite.draw_rect(
                    item_x, item_y, Self::WIDTH / Self::COLUMNS + column, Self::ROW_HEIGHT,
                    Colour::BLUE, ShapeFill::Filled, 0,
                );
            }

            // Draw item border
            self.os.display_sprite.draw_rect(
                item_x, item_y, Self::WIDTH / Self::COLUMNS + column, Self::ROW_HEIGHT,
                Colour::BLACK, ShapeFill::Hollow, 0,
            );

            // Draw item text
            self.os.display_sprite.print_at(
                item_x + Self::ITEM_PADDING as i16,
                item_y + Self::ITEM_PADDING as i16,
                &item.name
            );

            // Advance column and optionally increase row
            if column == 0 || column == 1 {
                column += 1;
            } else if column == 2 {
                column = 0;
                row += 1;
            } else {
                unreachable!()
            }
        }

        // Draw border around the whole thing - we do this at the end so the item borders don't
        // overwrite it
        self.os.display_sprite.draw_rect(
            starting_x, starting_y, Self::WIDTH - 1, Self::HEIGHT,
            Colour::WHITE, ShapeFill::Hollow, 0,
        );

        // Draw description panel
        let panel_y = starting_y + (Self::HEIGHT - Self::DESCRIPTION_HEIGHT) as i16;
        self.os.display_sprite.draw_rect(
            starting_x, panel_y,
            Self::WIDTH, Self::DESCRIPTION_HEIGHT,
            Colour::WHITE, ShapeFill::Hollow, 0,  
        );
        let description = self.selected().description.clone();
        let wrapped_text = self.os.display_sprite.wrap_text(
            &description,
            Self::WIDTH - Self::ITEM_PADDING * 2
        );
        for (i, line) in wrapped_text.0.iter().enumerate() {
            self.os.display_sprite.print_at(
                starting_x + Self::ITEM_PADDING as i16,
                panel_y + Self::ITEM_PADDING as i16 + wrapped_text.1 * i as i16,
                line,
            );
            if i == 2 { break; }
        }

        // Redraw
        self.os.draw();

        // Handle input
        match self.os.input() {
            Some(OSInput::Button(ButtonInput::MoveLeft)) => {
                // Move left by decrementing index, unless we're already at the extreme left (index
                // is a multiple of 3)
                if self.selected_index % 3 != 0 {
                    self.selected_index -= 1;
                }
            },
            Some(OSInput::Button(ButtonInput::MoveRight)) => {
                // Move right by incrementing index, unless we're already at the extreme right
                // (index mod 3 is 2), or this would take us off the end of the list
                if self.selected_index % 3 != 2 && self.selected_index + 1 < self.items.len() {
                    self.selected_index += 1;
                }
            },
            Some(OSInput::Button(ButtonInput::MoveUp)) => {
                // Move up by subtracting 3 from index, unless index is already less than 3
                if self.selected_index >= 3 {
                    self.selected_index -= 3;
                }
            },
            Some(OSInput::Button(ButtonInput::MoveDown)) => {
                // Move down by adding 3 to index, unless that would spill off the list
                if self.selected_index + 3 < self.items.len() {
                    self.selected_index += 3;
                }
            },
            Some(OSInput::Button(ButtonInput::Exe)) => {
                return SelectorMenuTickResult::Selected;
            },
            Some(OSInput::Button(ButtonInput::List)) => {
                return SelectorMenuTickResult::Cancelled;
            }

            _ => (),
        }

        SelectorMenuTickResult::Normal
    }
}
