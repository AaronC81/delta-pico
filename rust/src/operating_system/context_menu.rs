use alloc::{string::String, vec::Vec, boxed::Box};

use crate::{interface::{ApplicationFramework, DISPLAY_HEIGHT, DISPLAY_WIDTH, Colour, ShapeFill, ButtonInput}, operating_system::OSInput, graphics::Sprite};

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
    Divider,
}

impl<T> ContextMenuItem<T> {
    pub fn height(&self) -> u16 {
        match self {
            Self::Text { .. } => 30,
            Self::Divider => 8,
        }
    }

    pub fn draw(&self, sprite: &mut Sprite, y: i16) {
        match self {
            Self::Text { text, .. } => sprite.print_at(10, y + 4, text),
            Self::Divider => sprite.draw_line(5, y + 4, DISPLAY_WIDTH as i16 - 5, y + 4, Colour::WHITE),
        }
    }

    pub fn selectable(&self) -> bool {
        match self {
            Self::Text { .. } => true,
            Self::Divider => false,
        }
    }
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
            Self::Divider => panic!("tried to get inner of unselectable item"),
        }
    }

    fn into_inner(self) -> T {
        match self {
            Self::Text { metadata, .. } => metadata,
            Self::Divider => panic!("tried to get inner of unselectable item"),
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
        let total_height: u16 = self.items.iter().map(|i| i.height()).sum();

        // Draw background
        let mut y = (DISPLAY_HEIGHT - total_height - 20) as i16;
        self.os.display_sprite.draw_rect(0, y, DISPLAY_WIDTH, 400, Colour::GREY, ShapeFill::Filled, 10);
        self.os.display_sprite.draw_rect(0, y, DISPLAY_WIDTH, 400, Colour::WHITE, ShapeFill::Hollow, 10);

        // Draw items
        y += 10;
        for (i, item) in self.items.iter().enumerate() {
            // Calculate height of this item
            let height = item.height();

            // If it's selected, draw a background
            if i == self.selected_index {
                let width = DISPLAY_WIDTH;
                self.os.display_sprite.draw_rect(
                    5, y as i16, width - 5 * 2, height,
                    Colour::BLUE, ShapeFill::Filled, 7
                );
            }
            
            // Draw item
            item.draw(&mut self.os.display_sprite, y);

            // Move up by height
            y += height as i16;
        }

        // Redraw
        self.os.draw();

        // Handle input
        if let Some(btn) = self.os.input() {
            match btn {
                OSInput::Button(ButtonInput::MoveUp) => {
                    loop {
                        if self.selected_index == 0 {
                            self.selected_index = self.items.len() - 1;
                        } else {
                            self.selected_index -= 1;
                        }

                        if self.items[self.selected_index].selectable() { break }
                    }
                }
                OSInput::Button(ButtonInput::MoveDown) => {
                    loop {
                        self.selected_index += 1;
                        self.selected_index %= self.items.len();

                        if self.items[self.selected_index].selectable() { break }
                    }
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
