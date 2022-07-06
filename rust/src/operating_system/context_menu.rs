use alloc::{string::String, vec::Vec, boxed::Box};

use crate::{interface::{ApplicationFramework, DISPLAY_HEIGHT, DISPLAY_WIDTH, Colour, ShapeFill, ButtonInput}, operating_system::OSInput};

use super::{OperatingSystemPointer, SelectorMenu, SelectorMenuTickResult, SelectorMenuItem};


pub struct ContextMenu<F, T>
where F: ApplicationFramework + 'static
{
    os: OperatingSystemPointer<F>,
    items: Vec<ContextMenuItem<T>>,
    selected_index: usize,
    closeable: bool,
}

pub enum ContextMenuItem<T> {
    Text {
        text: String,
        metadata: T,
    },
}

impl<A> ContextMenuItem<Box<dyn FnOnce(&mut A)>> {
    pub fn new_common(text: impl Into<String>, func: impl FnOnce(&mut A) + 'static) -> Self {
        ContextMenuItem::Text {
            text: text.into(),
            metadata: Box::new(func),
        }
    }
}

impl<T> SelectorMenuItem for ContextMenuItem<T> {
    type Inner = T;

    fn inner(&self) -> &T {
        match self {
            Self::Text { metadata, .. } => metadata,
        }
    }

    fn into_inner(self) -> T {
        match self {
            Self::Text { metadata, .. } => metadata,
        }
    }
}

impl<F, T> ContextMenu<F, T>
where F: ApplicationFramework + 'static
{
    pub fn new(os: OperatingSystemPointer<F>, items: Vec<ContextMenuItem<T>>, closeable: bool) -> Self {
        Self {
            os,
            items,
            closeable,
            selected_index: 0,
        }
    }
}

impl<F, T> SelectorMenu for ContextMenu<F, T>
where F: ApplicationFramework + 'static,
{
    type Item = ContextMenuItem<T>;

    fn selected_index(&self) -> usize { self.selected_index }
    fn items(&self) -> &Vec<Self::Item> { &self.items }
    fn into_items(self) -> Vec<Self::Item> { self.items }

    fn tick(&mut self) -> SelectorMenuTickResult {
        const ITEM_GAP: i16 = 30;

        // Draw background
        let mut y = DISPLAY_HEIGHT as i16 - ITEM_GAP * self.items.len() as i16 - 10;
        self.os.display_sprite.draw_rect(0, y, DISPLAY_WIDTH, 400, Colour::GREY, ShapeFill::Filled, 10);
        self.os.display_sprite.draw_rect(0, y, DISPLAY_WIDTH, 400, Colour::WHITE, ShapeFill::Hollow, 10);

        // Draw items
        y += 10;
        for (i, item) in self.items.iter().enumerate() {
            if i == self.selected_index {
                let width = DISPLAY_WIDTH;
                self.os.display_sprite.draw_rect(
                    5, y as i16, width - 5 * 2, 25,
                    Colour::BLUE, ShapeFill::Filled, 7
                );
            }
            self.os.display_sprite.print_at(10, y + 4, match item {
                ContextMenuItem::Text { text, .. } => &text,
            });

            y += ITEM_GAP;
        }

        // Redraw
        self.os.draw();

        // Handle input
        if let Some(btn) = self.os.input() {
            match btn {
                OSInput::Button(ButtonInput::MoveUp) => {
                    if self.selected_index == 0 {
                        self.selected_index = self.items.len() - 1;
                    } else {
                        self.selected_index -= 1;
                    }
                }
                OSInput::Button(ButtonInput::MoveDown) => {
                    self.selected_index += 1;
                    self.selected_index %= self.items.len();
                }
                OSInput::Button(ButtonInput::Exe) => {
                    return SelectorMenuTickResult::Selected;
                }
                OSInput::Button(ButtonInput::List) if self.closeable => {
                    return SelectorMenuTickResult::Cancelled;
                }
                _ => (),
            }
        }

        SelectorMenuTickResult::Normal
    }
}
