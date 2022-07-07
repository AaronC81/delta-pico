use alloc::{format, vec::Vec, vec};
use az::SaturatingAs;
use rbop::{render::{Viewport, Area}, node::unstructured::{UnstructuredNodeRoot, Upgradable}, Number, nav::NavPath};

use crate::{interface::{ApplicationFramework, Colour, ShapeFill, DISPLAY_WIDTH, ButtonInput}, operating_system::{OSInput, OperatingSystemPointer}, rbop_impl::{RbopContext, RbopSpriteRenderer}, applications::calculator::{catalog::Catalog, CalculatorApplication}};

use super::OperatingSystem;

impl<F: ApplicationFramework + 'static> OperatingSystem<F> {
    /// Draws a title bar to the top of the screen, with the text `s`.
    pub fn ui_draw_title(&mut self, s: &str) {
        self.display_sprite.draw_rect(
            0, 0, self.display_sprite.width, Self::TITLE_BAR_HEIGHT,
            Colour::ORANGE, ShapeFill::Filled, 0
        );
        self.display_sprite.print_at(5, 7, &s);

        // Draw charge indicator
        // let charge_status = (framework().charge_status)();
        // let charge_bitmap = if charge_status == -1 { "power_usb".into() } else { format!("battery_{}", charge_status) };
        // self.framework.display().draw_bitmap(200, 6, &charge_bitmap);

        // Draw text indicator
        if self.text_mode {
            self.display_sprite.draw_rect(145, 4, 50, 24, Colour::WHITE, ShapeFill::Hollow, 5);
            if self.input_shift {
                self.display_sprite.print_at(149, 6, "TEXT");
            } else {
                self.display_sprite.print_at(153, 6, "text");
            }
        }
    }

    /// Opens an rbop input box with the given `title` and optionally starts the node tree at the
    /// given `root`. When the user presses EXE, returns the current node tree.
    pub fn ui_input_expression(&mut self, title: &str, root: Option<UnstructuredNodeRoot>) -> UnstructuredNodeRoot {
        const PADDING: i16 = 10;
        
        let mut rbop_ctx = RbopContext {
            viewport: Some(Viewport::new(Area::new(
                (self.display_sprite.width - PADDING as u16 * 2).into(),
                (self.display_sprite.height - PADDING as u16 * 2).into(),
            ))),
            ..RbopContext::<F>::new(self.ptr)
        };

        // If we've been given an existing root to use, then set that and move the cursor to the end
        if let Some(unr) = root {
            rbop_ctx.root = unr;
            rbop_ctx.nav_path = NavPath::new(vec![rbop_ctx.root.root.items.len()]);
        }

        // Don't let the box get any shorter than the maximum height it has achieved, or you'll get
        // ghost boxes if the height reduces since we don't redraw the whole frame
        let mut minimum_height = 0u16;
    
        loop {
            // Draw expression to sprite
            let sprite = RbopSpriteRenderer::draw_context_to_sprite(&mut rbop_ctx, Colour::GREY);
            
            if sprite.height > minimum_height {
                minimum_height = sprite.height;
            }

            // Draw background of dialog
            let y = self.display_sprite.height
                - minimum_height
                - 30
                - PADDING as u16 * 2;
            self.display_sprite.draw_rect(0, y.saturating_as::<i16>(), DISPLAY_WIDTH, 400, Colour::GREY, ShapeFill::Filled, 10);
            self.display_sprite.draw_rect(0, y.saturating_as::<i16>(), DISPLAY_WIDTH, 400, Colour::WHITE, ShapeFill::Hollow, 10);      
            
            // Draw title
            self.display_sprite.print_at(PADDING, y.saturating_as::<i16>() + PADDING, title);

            // Draw expression sprite to display
            self.display_sprite.draw_sprite(PADDING, y as i16 + 30 + PADDING, &sprite);

            // Update screen
            self.draw();

            // Poll for input
            if let Some(input) = self.input() {
                match input {
                    OSInput::Button(ButtonInput::Exe) => return rbop_ctx.root,
                    OSInput::Button(ButtonInput::List) => {
                        // TODO: should this open a list for consistency with Calculator?

                        // We won't be redrawing the whole app, just the expression input dialog, so
                        // this catalog wouldn't get redrawn, which looks very odd. Instead, take a
                        // copy of the display sprite right now. We can then restore the copy later,
                        // once the catalog is closed.
                        // (Very memory intensive! But a display sprite is ~70kB, 240kB available,
                        // and I've rarely seen more than 100kB used, so should be fine.)
                        let display_sprite_before_catalog = self.display_sprite.clone();

                        let catalog = Catalog::new(
                            OperatingSystemPointer::new(self as *mut _),
                            "Catalog",
                            CalculatorApplication::<F>::catalog_items(),
                        );
                        if let Some(item) = catalog.tick_until_complete() {
                            rbop_ctx.root.insert(
                                &mut rbop_ctx.nav_path,
                                &mut RbopSpriteRenderer::new(),
                                rbop_ctx.viewport.as_mut(),
                                item.metadata,
                            );
                        }

                        self.display_sprite = display_sprite_before_catalog;
                    }
                    _ => {
                        rbop_ctx.input(input);
                    }
                }
            }
        }
    }

    /// A variant of `ui_input_expression` which upgrades and evaluates the input.
    /// If this causes an error, a dialog will be displayed with `ui_text_dialog`, which will
    /// require redrawing the screen once dismissed. As such, this takes a `redraw` function which
    /// will be called each time before displaying the input prompt (including the first time).
    pub fn ui_input_expression_and_evaluate(
        &mut self,
        title: &str,
        root: Option<UnstructuredNodeRoot>,
        mut redraw: impl FnMut(),
    ) -> (Number, UnstructuredNodeRoot) {
        let mut unr = root;
        loop {
            redraw();
            unr = Some(self.ui_input_expression(title, unr));
            match unr
                .as_ref()
                .unwrap()
                .upgrade()
                .map_err(|e| format!("{:?}", e))
                .and_then(|sn| sn
                    .evaluate(&self.filesystem.settings.evaluation_settings())
                    .map_err(|e| format!("{:?}", e))) {
                
                Ok(d) => {
                    return (d.simplify(), unr.unwrap());
                }
                Err(s) => {
                    redraw();
                    self.ui_text_dialog(&s);
                }
            }
        }
    }

    /// Opens a text dialog in the centre of the screen which can be dismissed with EXE.
    pub fn ui_text_dialog(&mut self, s: &str) {
        const H_PADDING: u16 = 30;
        const H_INNER_PADDING: u16 = 10;
        const V_PADDING: u16 = 10;

        let w = self.display_sprite.width - H_PADDING * 2;
        let (lines, ch, h) = self.display_sprite.wrap_text(s, w - H_INNER_PADDING * 2);
        let h = h as u16;
        let y_start = (self.display_sprite.height - h) / 2;

        self.display_sprite.draw_rect(
            H_PADDING as i16, y_start as i16,
            w, h + V_PADDING as u16 * 2,
            Colour::GREY, ShapeFill::Filled, 10
        );
        self.display_sprite.draw_rect(
            H_PADDING as i16, y_start as i16,
            w, h + V_PADDING as u16 * 2,
            Colour::WHITE, ShapeFill::Hollow, 10
        );
        
        for (i, line) in lines.iter().enumerate() {
            self.display_sprite.print_at(
                (H_PADDING + H_INNER_PADDING) as i16,
                y_start as i16 + V_PADDING as i16 + ch as i16 * i as i16,
                line
            );
        }

        // Push to screen
        self.draw();

        // Poll for input
        loop {
            if let Some(input) = self.input() {
                if OSInput::Button(ButtonInput::Exe) == input {
                    break;
                }
            }
        }
    }
}

/// The result of `SelectorMenu::tick`, indicating whether any external handling needs to occur.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SelectorMenuTickResult {
    /// The tick completed with no further external action required.
    Normal,

    /// The user selected an item in the menu with the EXE key.
    Selected,

    /// The user dismissed the menu with the LIST key.
    Cancelled,
}

pub trait SelectorMenu
where Self: Sized
{
    type Item: SelectorMenuItem;

    fn selected_index(&self) -> usize;
    fn items(&self) -> &Vec<Self::Item>;
    fn into_items(self) -> Vec<Self::Item>;

    fn tick(&mut self) -> SelectorMenuTickResult;

    /// Repeatedly draws the menu to the screen and takes a key of input, until either:
    ///   - An item is selected, which returns `Some` with the selected item
    ///   - The menu is dismissed, which returns `None`.
    fn tick_until_complete(mut self) -> Option<Self::Item> {
        let mut last_result;
        loop {
            last_result = self.tick();
            match last_result {
                SelectorMenuTickResult::Normal => continue,
                SelectorMenuTickResult::Selected => return Some(self.into_selected()),
                SelectorMenuTickResult::Cancelled => return None,
            }
        }
    }

    /// Retrieves a reference to the item currently hovered for selection.
    fn selected(&self) -> &Self::Item {
        &self.items()[self.selected_index()]
    }

    /// Consumes the menu and returns the item hovered for selection.
    fn into_selected(self) -> Self::Item {
        // Mutation won't have unintended consequences because this method consumes and drops `self`
        // anyway
        let index = self.selected_index();
        self.into_items().remove(index)
    }
}

/// An item in a selector menu.
pub trait SelectorMenuItem {
    type Inner;

    fn inner(&self) -> &Self::Inner;
    fn into_inner(self) -> Self::Inner;
}

/// A convenience extension trait which is implemented for `SelectorMenu`s where `Item::Inner` is
/// a function taking one argument. This enables easier writing of menus which call functions when
/// an item is selected.
pub trait SelectorMenuCallable<A, R>: SelectorMenu
{
    fn tick_until_call(self, arg: A) -> Option<R>;
}

impl<X, XI, F, A, R> SelectorMenuCallable<A, R> for X
where
    X: SelectorMenu<Item = XI>,
    XI: SelectorMenuItem<Inner = F>,
    F: FnOnce(A) -> R
{
    /// The same as `tick_until_complete`, but calls the selected item, and returns the resulting
    /// value.
    /// 
    /// The extra argument `arg` is passed to the called function, and is designed to be used to
    /// pass `self` without the closures having to capture it (which the borrow checker would
    /// disallow).
    fn tick_until_call(self, arg: A) -> Option<R> {
        self.tick_until_complete().map(|x| (x.into_inner())(arg))
    }
}
