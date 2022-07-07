use core::cmp::{max, min};
use alloc::{format, vec, vec::Vec};
use rbop::{Number, StructuredNode, nav::MoveVerticalDirection, node::{unstructured::{MoveResult, Upgradable}, function::Function}, render::{Area, Renderer, Viewport, LayoutComputationProperties}, UnstructuredNode, UnstructuredNodeList, Token};

use crate::{filesystem::{Calculation, ChunkIndex, CalculationResult}, interface::{Colour, ApplicationFramework, DisplayInterface, ButtonInput, ShapeFill, DISPLAY_WIDTH}, operating_system::{OSInput, OperatingSystem, os_accessor, OperatingSystemPointer, SelectorMenu, ContextMenu, ContextMenuItem, SelectorMenuCallable}, rbop_impl::{RbopContext, RbopSpriteRenderer}, graphics::Sprite};
use self::catalog::{CatalogItem, Catalog};

use super::{Application, ApplicationInfo};

mod sprite_cache;
use sprite_cache::*;

mod test;

pub mod catalog;

const PADDING: u64 = 10;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum Selection {
    Expression(usize),
    Result(usize),
}

impl Selection {
    fn index(&self) -> usize {
        match *self {
            Selection::Expression(i) => i,
            Selection::Result(i) => i,
        }
    }

    fn down(self) -> Selection {
        match self {
            Selection::Expression(i) => Selection::Result(i),
            Selection::Result(i) => Selection::Expression(i + 1),
        }
    }

    fn up(self) -> Selection {
        match self {
            Selection::Expression(i) => Selection::Result(i - 1),
            Selection::Result(i) => Selection::Expression(i),
        }
    }
}

pub struct CalculatorApplication<F: ApplicationFramework + 'static> {
    os: OperatingSystemPointer<F>,

    calculations: Vec<Calculation>,
    selection: Selection,
    rbop_ctx: RbopContext<F>,

    /// The sprite cache; see the docs for `SpriteCache`.
    sprite_cache: SpriteCache,

    /// The Y which we starting drawing calculations decreasing from (that is, drawing them up the
    /// screen.) A non-scrolled screen should have a starting_y of the screen's height, since we'll
    /// begin drawing up from the bottom of the screen.
    starting_y: i16,

    /// If the current selection is a result, the amount by which it is scrolled. 0 means the
    /// beginning is on the left of the screen and it may spill off the right, and any value above
    /// subtracts from the starting X, gradually spilling it off the left.
    result_scroll_x: u16,
}

os_accessor!(CalculatorApplication<F>);

impl<F: ApplicationFramework> Application for CalculatorApplication<F> {
    type Framework = F;

    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "Calculator".into(),
            visible: true,
        }
    }

    fn new(mut os: OperatingSystemPointer<F>) -> Self {
        let mut calculations = if let Some(c) = os.filesystem.calculations.read_calculations() {
            c
        } else {
            os.ui_text_dialog("Failed to load calculation history.");
            vec![]
        };
        
        // Add empty calculation onto the end if it is not already empty, or if there are no
        // calculations at all
        let needs_empty_adding = if let Some(Calculation { root, .. }) = calculations.last() {
            !root.root.items.is_empty()
        } else {
            true
        };
        if needs_empty_adding {
            calculations.push(Calculation::blank());
        }

        let selection = Selection::Expression(calculations.len() - 1);
        let root = calculations[selection.index()].root.clone();
        
        let mut result = Self {
            os,
            rbop_ctx: RbopContext {
                viewport: Some(Viewport::new(Area::new(
                    (os.display_sprite.width - PADDING as u16 * 2).into(),
                    (os.display_sprite.height - PADDING as u16 * 2).into(),
                ))),
                root,
                ..RbopContext::new(os)
            },
            calculations,
            selection,
            sprite_cache: SpriteCache::new(),
            starting_y: os.framework.display().height() as i16,
            result_scroll_x: 0,
        };
        result.sprite_cache.clear(result.calculations.len());
        result
    }

    fn tick(&mut self) {
        // Clear screen
        self.os_mut().display_sprite.fill(Colour::BLACK);

        // We draw the history vec in reverse, starting with the last item. To make this easier, we
        // also draw the screen from bottom to top. 
        let mut next_calculation_highest_y = self.starting_y;
        
        // Draw history
        let calculation_count = self.calculations.len();
        let mut rest_are_clipped = false;
        let mut selected_result_width = None;
        for i in (0..self.calculations.len()).rev() {
            // If the last thing we drew was partially off the top of the screen, then this is fully
            // off the screen, so skip it and mark it as pruned
            if rest_are_clipped {
                self.sprite_cache[i] = SpriteCacheEntry::ClippedOffTop;
                
                // TODO: Is this OK, or is there some reason it was missing until very recently?
                continue;
            }

            // Fetch this from the sprite cache
            self.sprite_cache.create_if_blank(i, &mut self.calculations);
            let mut calculation_sprite = match self.sprite_cache.entry_data(i) {
                Some(x) => x,
                // If clipped, we don't need to draw this
                None => continue,
            };

            // Lay out this node, so we can work out height
            // We'll also calculate a result here since we might as well
            let result_height;
            let mut result_sprite;
            let mut new_calculation_sprite; // In case we need to allocate a new sprite
            
            if let SpriteCacheEntryData::Height { result: cached_result_height, .. } = calculation_sprite {
                // This is entirely off the screen, so we only care about the height for layout
                // purposes
                result_height = *cached_result_height;
                result_sprite = None;
            } else {
                let result;
                if self.selection == Selection::Expression(i) {
                    // If this is the calculation currently being edited, there is a possibly edited
                    // version in the rbop context, so use that instead of the cached sprite and result
                    result = match self.rbop_ctx.root.upgrade() {
                        Ok(structured) => match structured.evaluate(&self.os().filesystem.settings.evaluation_settings()) {
                            Ok(evaluation_result) => CalculationResult::Ok(evaluation_result.simplify()),
                            Err(err) => CalculationResult::MathsError(err),
                        },
                        Err(err) => CalculationResult::NodeError(err),
                    };

                    new_calculation_sprite = SpriteCacheEntryData::Sprite(
                        RbopSpriteRenderer::draw_context_to_sprite(&mut self.rbop_ctx, Colour::BLACK)
                    );
                    calculation_sprite = &mut new_calculation_sprite;
                } else {
                    result = self.calculations[i].result.clone()
                };

                // Draw sprite for result
                let result_bg_colour = if self.selection == Selection::Result(i) {
                    Colour::GREY
                } else {
                    Colour::BLACK
                };
                result_sprite = Some(Self::draw_result_to_sprite(&result, result_bg_colour));
                result_height = PADDING as u16 * 3 + result_sprite.as_ref().unwrap().height;
            }

            // Work out Y position to draw everything from. Since we draw from bottom to top, we
            // need to subtract the height of what we're drawing from base Y
            let calculation_height = match calculation_sprite {
                SpriteCacheEntryData::Height { calculation, .. } => *calculation,
                SpriteCacheEntryData::Sprite(s) => s.height,
            };
            let this_calculation_lowest_y =
                // Global start
                next_calculation_highest_y - (
                    // Node
                    calculation_height + PADDING as u16 +
                    // Result
                    result_height
                ) as i16;

            // If the lowest Y is off the top of the screen (it could still be partially visible)...
            if this_calculation_lowest_y < OperatingSystem::<F>::TITLE_BAR_HEIGHT as i16 {
                // Then everything else is off the screen
                rest_are_clipped = true;

                // Is this the current calculation? If so, we've scrolled up but this was off the
                // screen, and we need to adjust the starting Y to show this entire calculation 
                if self.selection.index() == i {
                    // TODO: this does a weirdly large scroll if the calculation at the top of the
                    // screen gets taller

                    // The amount which is off the screen (not including the title bar) happens to
                    // be abs(this_calculation_lowest_y), so we can scroll by that amount by adding
                    // it to the starting Y...
                    self.starting_y += this_calculation_lowest_y.abs();

                    // But we also need to account for that title bar!
                    self.starting_y += OperatingSystem::<F>::TITLE_BAR_HEIGHT as i16;

                    // Redraw
                    self.tick();
                    return;
                }
            }

            // If the greatest Y is off the bottom of the screen (again, maybe still partially
            // visible) and this is the calculation we're currently editing...
            if next_calculation_highest_y > self.os().display_sprite.height as i16
                && self.selection.index() == i
            {
                // We need to scroll down by the different between the height of the display and the
                // highest Y
                self.starting_y -= next_calculation_highest_y - self.os().display_sprite.height as i16;
                
                // Redraw
                self.tick();
                return;
            }

            // The next calculation, drawn above this one, should have its highest Y be the same as
            // the lowest Y of this equation, minus one so they don't overlap
            next_calculation_highest_y = this_calculation_lowest_y - 1;
            
            // If the lowest Y is off the bottom of the screen, then this isn't shown on the screen
            // at all
            if this_calculation_lowest_y > self.os().display_sprite.height as i16 {
                // If the sprite cache entry contains a sprite, we can replace it with just height
                if let SpriteCacheEntryData::Sprite(s) = calculation_sprite {
                    self.sprite_cache[i] = SpriteCacheEntry::Entry {
                        data: SpriteCacheEntryData::Height {
                            calculation: s.height,
                            result: result_height,
                        }
                    };

                    // We *just* set this index, so only the `Entry` case could ever occur
                    calculation_sprite = match self.sprite_cache[i] {
                        SpriteCacheEntry::Entry { ref data } => data,
                        _ => unreachable!(),
                    };
                }
            }
            
            // Draw calculation sprite (if we kept the sprite)
            if let SpriteCacheEntryData::Sprite(sprite) = calculation_sprite {
                self.os_mut().display_sprite.draw_sprite(
                    PADDING as i16,
                    this_calculation_lowest_y + PADDING as i16,
                    sprite,
                );
            }

            // As we draw different components of the calculation, we'll add to the current Y
            // accordingly.
            let mut this_calculation_current_y = this_calculation_lowest_y;
            this_calculation_current_y += calculation_height as i16 + PADDING as i16;
            
            // Draw result
            if let Some(result_sprite) = &mut result_sprite {
                let is_this_result_selected = self.selection == Selection::Result(i);
                if is_this_result_selected {
                    selected_result_width = Some(result_sprite.width);   
                }
                self.draw_result(this_calculation_current_y, result_sprite, is_this_result_selected);
            }
            this_calculation_current_y += result_height as i16;

            // Draw a big line, unless this is the last item
            if i != calculation_count - 1 {
                self.os_mut().display_sprite.draw_line(
                    0, this_calculation_current_y,
                    self.os().display_sprite.width as i16, this_calculation_current_y,
                    Colour::WHITE,
                )
            }
        }

        // Write title
        self.os_mut().ui_draw_title("Calculator");

        // Push to screen
        self.os_mut().draw();

        // Poll for input
        if let Some(input) = self.os_mut().input() {
            if input == OSInput::Button(ButtonInput::Exe) {
                // Save whatever we're editing
                self.save_current();

                // Add a new calculation to the end of the list
                self.calculations.push(Calculation::blank());

                // Move to it
                self.selection = Selection::Expression(self.calculations.len() - 1);
                self.reset_scroll();

                // Reset the rbop context, and save the new calculation
                self.load_current();  
                self.save_current();

                // Clear the sprite cache
                self.sprite_cache.clear(self.calculations.len());
            } else if input == OSInput::Button(ButtonInput::List) {
                ContextMenu::new(
                    self.os,
                    vec![
                        ContextMenuItem::new_common("Catalog...", |this: &mut Self| {
                            let catalog = Catalog::new(this.os, "Catalog", Self::catalog_items());
                            if let Some(item) = catalog.tick_until_complete() {
                                let node = item.metadata;

                                this.rbop_ctx.root.insert(
                                    &mut this.rbop_ctx.nav_path,
                                    &mut RbopSpriteRenderer::new(),
                                    this.rbop_ctx.viewport.as_mut(),
                                    node,
                                );
                            }
                        }),

                        ContextMenuItem::new_common("Clear history", |this: &mut Self| {
                            // Delete from storage
                            this.os_mut().filesystem.calculations.table.clear(false);
                            
                            // There are too many things to reload manually, just restart the app
                            return this.os_mut().restart_application();
                        })
                    ],
                    true,
                ).tick_until_call(self);
            } else if matches!(self.selection, Selection::Result(_)) && matches!(input, OSInput::Button(ButtonInput::MoveLeft | ButtonInput::MoveRight)) {
                match input {
                    OSInput::Button(ButtonInput::MoveLeft) => self.result_scroll_x -= min(self.result_scroll_x, 10),
                    OSInput::Button(ButtonInput::MoveRight) => {
                        let selected_result_width = selected_result_width.expect("unknown result width");
                        if selected_result_width > DISPLAY_WIDTH - PADDING as u16 * 2 {
                            self.result_scroll_x += 10;
                            let max_scroll = selected_result_width - DISPLAY_WIDTH + PADDING as u16 * 2;
                            if self.result_scroll_x > max_scroll {
                                self.result_scroll_x = max_scroll;
                            }
                        }
                    },
                    _ => (),
                }
            } else {
                let move_result = self.rbop_ctx.input(input);
                // Move calculations if needed
                if let Some((dir, MoveResult::MovedOut)) = move_result {
                    match dir {
                        MoveVerticalDirection::Up => if self.selection != Selection::Expression(0) {
                            self.save_current();
                            self.selection = self.selection.up();
                            self.load_current();
                            self.sprite_cache.clear(self.calculations.len());
                        },
                        MoveVerticalDirection::Down => if self.selection != Selection::Result(self.calculations.len() - 1) {
                            self.save_current();
                            self.selection = self.selection.down();
                            self.result_scroll_x = 0;
                            self.load_current();
                            self.sprite_cache.clear(self.calculations.len());
                        },
                    }
                }
            }
        }
    }

    fn test(&mut self) {
        test::test(self);
    }
}

impl<F: ApplicationFramework> CalculatorApplication<F> {
    fn save_current(&mut self) {
        // Evaluate
        let result = match self.rbop_ctx.root.upgrade() {
            Ok(structured) => match structured.evaluate(&self.os().filesystem.settings.evaluation_settings()) {
                Ok(evaluation_result) => CalculationResult::Ok(evaluation_result.simplify()),
                Err(err) => CalculationResult::MathsError(err),
            },
            Err(err) => CalculationResult::NodeError(err),
        };

        // Save into array
        self.calculations[self.selection.index()].root = self.rbop_ctx.root.clone();
        self.calculations[self.selection.index()].result = result;

        // Save to storage
        self.os_mut().filesystem.calculations.write_calculation_at_index(
            ChunkIndex(self.selection.index() as u16),
            self.calculations[self.selection.index()].clone()
        );
    }
    
    fn load_current(&mut self) {
        // Reset rbop context
        self.rbop_ctx = RbopContext {
            viewport: Some(Viewport::new(Area::new(
                self.os().display_sprite.width as u64 - PADDING * 2,
                self.os().display_sprite.height as u64 - PADDING * 2,
            ))),
            root: self.calculations[self.selection.index()].root.clone(),
            ..RbopContext::new(self.os)
        };
    }
    
    fn draw_result(&mut self, y: i16, result_sprite: &mut Sprite, is_selected: bool) {
        if is_selected {
            self.os_mut().display_sprite.draw_rect(
                0, y + PADDING as i16,
                self.os_mut().display_sprite.width as u16, 2 * PADDING as u16 + result_sprite.height,
                Colour::GREY, ShapeFill::Filled, 0
            );
        }

        // Draw a line
        self.os_mut().display_sprite.draw_line(
            PADDING as i16, y + PADDING as i16,
            self.os_mut().display_sprite.width as i16 - PADDING as i16, y + PADDING as i16,
            Colour::GREY
        );

        // Draw the result sprite right-aligned
        self.os_mut().display_sprite.draw_sprite(
            max((self.os().display_sprite.width as i16 - PADDING as i16) - result_sprite.width as i16, PADDING as i16)
            - if is_selected { self.result_scroll_x as i16 } else { 0 },
            y + PADDING as i16 * 2,
            result_sprite,
        );
    }

    fn draw_result_to_sprite(result: &CalculationResult, background_colour: Colour) -> Sprite {        
        let error_string = match result {
            CalculationResult::Ok(number) => {
                // Convert the result number into a structured node
                let mut result_node = StructuredNode::Number(*number);

                // Render this node to a sprite
                return RbopSpriteRenderer::draw_to_sprite::<_>(
                    &mut result_node,
                    None,
                    None,
                    background_colour,
                );
            },

            CalculationResult::MathsError(err) => format!("{}", err),
            CalculationResult::NodeError(err) => format!("{}", err),

            CalculationResult::None => return Sprite::empty(),
        };

        // That `match` didn't return, create a sprite with an error string
        let (width, _) = Sprite::empty().font.string_size(&error_string);

        // We'll use the same height as a digit to avoid wobble when the result is flickering
        // between a number and an error
        let mut renderer = RbopSpriteRenderer::new();
        let height = renderer.layout(
            &StructuredNode::Number(Number::Rational(6, 1)),
            None,
            LayoutComputationProperties::default()
        ).area.height + 3;

        let mut sprite = Sprite::new(width as u16, height as u16);
        sprite.fill(background_colour);
        sprite.print_at(0, 0, &error_string);
        sprite
    }

    fn reset_scroll(&mut self) {
        self.starting_y = self.os().display_sprite.height as i16;
        self.result_scroll_x = 0;
    }

    pub fn catalog_items() -> Vec<CatalogItem<UnstructuredNode>> {
        vec![
            CatalogItem::new("x", "Variable for graph plots", UnstructuredNode::Token(Token::Variable('x'))),
            CatalogItem::new("sqrt", "Compute square root of a value", UnstructuredNode::Sqrt(UnstructuredNodeList::new())),
            CatalogItem::new("pow", "Raise a value to a power", UnstructuredNode::Power(UnstructuredNodeList::new())),
            CatalogItem::new("sin", "Trigonometric sine", UnstructuredNode::new_function_call(Function::Sine)),
            CatalogItem::new("cos", "Trigonometric cosine", UnstructuredNode::new_function_call(Function::Cosine)),
            CatalogItem::new("gcd", "Greatest common denominator of two values", UnstructuredNode::new_function_call(Function::GreatestCommonDenominator)),
        ]
    }
}
