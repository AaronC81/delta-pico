use alloc::{format, vec, vec::Vec};
use rbop::{Number, StructuredNode, nav::MoveVerticalDirection, node::{unstructured::{MoveResult, Upgradable}}, render::{Area, Renderer, Viewport, LayoutComputationProperties}};
use rust_decimal::{Decimal, prelude::Zero};

use crate::{filesystem::{Calculation, ChunkIndex, CalculationResult}, interface::{Colour, Sprite}, operating_system::{OSInput, OperatingSystemInterface, os}, rbop_impl::RbopContext, timer::Timer};
use super::{Application, ApplicationInfo};
use crate::interface::framework;

const PADDING: u64 = 10;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
/// An entry into the sprite cache.
enum SpriteCacheEntry {
    /// The sprite cache has been cleared, and this item hasn't been recomputed yet.
    Blank,

    /// This item was found to be completely off the screen, so has been marked as clipped. This
    /// item does not need to be drawn.
    Clipped,

    /// This item has been recomputing since the sprite cache was last cleared, and is at least
    /// partially visible on the screen.
    Entry { area: Area, sprite: Sprite },
}

pub struct CalculatorApplication {
    calculations: Vec<Calculation>,
    current_calculation_idx: usize,
    rbop_ctx: RbopContext,

    /// The sprite cache is an optimization technique which sacrifices memory in order to gain a
    /// significant performance boost. Computing and drawing an rbop layout is relatively expensive,
    /// so the sprite cache is used to lay out and draw the calculations which we are not editing
    /// onto sprites in advance. Calculations not being edited won't change unless we navigate
    /// between calculations, so these will be stored until the edited calculation changes. Drawing
    /// the sprites onto the screen is significantly faster than recomputing and redrawing the rbop
    /// layout.
    ///
    /// Supposing that there are 4 calculations on the screen, one of which is being edited:
    ///   - Without the sprite cache, every `tick` computes and draws 4 rbop layouts.
    ///   - With the sprite cache:
    ///      - The first `tick` after navigating between calculations computes and draws 4 rbop
    ///        layouts, allocates sprites for them, and performs a pass to mark other calculations
    ///        as off-screen.
    ///      - Every subsequent `tick` draws 3 sprites (negligible time) and 1 rbop layout.
    sprite_cache: Vec<SpriteCacheEntry>,

    show_timing: bool,

    /// The Y which we starting drawing calculations decreasing from (that is, drawing them up the
    /// screen.) A non-scrolled screen should have a starting_y of the screen's height, since we'll
    /// begin drawing up from the bottom of the screen.
    starting_y: i64,
}

impl Application for CalculatorApplication {
    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "Calculator".into(),
            visible: true,
        }
    }

    fn new() -> Self {
        let mut calculations = if let Some(c) = os().filesystem.calculations.read_calculations() {
            c
        } else {
            os().ui_text_dialog("Failed to load calculation history.");
            vec![]
        };
        
        // Add empty calculation onto the end if it is not already empty, or if there are no
        // calculations at all
        let needs_empty_adding = if let Some(Calculation { root, .. }) = calculations.last() {
            if root.root.items.is_empty() {
                false
            } else {
                true
            }
        } else {
            true
        };
        if needs_empty_adding {
            calculations.push(Calculation::blank());
        }

        let current_calculation_idx = calculations.len() - 1;
        let root = calculations[current_calculation_idx].root.clone();
        
        let mut result = Self {
            rbop_ctx: RbopContext {
                viewport: Some(Viewport::new(Area::new(
                    framework().display.width - PADDING * 2,
                    framework().display.height - PADDING * 2,
                ))),
                root,
                ..RbopContext::new()
            },
            calculations,
            current_calculation_idx,
            sprite_cache: vec![],
            show_timing: false,
            starting_y: framework().display.height as i64,
        };
        result.clear_sprite_cache();
        result
    }

    fn tick(&mut self) {
        // Clear screen
        framework().display.fill_screen(Colour::BLACK);

        let _result_string_height = framework().display.string_size("A").1;

        let mut top_level_timer = Timer::new("Tick");
        top_level_timer.start();
        let height_timer = top_level_timer.new_subtimer("Height");
        let layout_timer = height_timer.borrow_mut().new_subtimer("Layout");
        let eval_timer = height_timer.borrow_mut().new_subtimer("Eval");
        let area_timer = height_timer.borrow_mut().new_subtimer("Area calc");

        let draw_node_timer = top_level_timer.new_subtimer("Draw node");
        let draw_result_timer = top_level_timer.new_subtimer("Draw result");

        // We draw the history vec in reverse, starting with the last item. To make this easier, we
        // also draw the screen from bottom to top. 
        let mut next_calculation_highest_y = self.starting_y;
        
        // Draw history
        let calculation_count = self.calculations.len();
        let mut rest_are_clipped = false;
        for i in (0..self.calculations.len()).rev() {
            // If the last thing we drew was partially off the top of the screen, then this is fully
            // off the screen, so skip it and mark it as pruned
            if rest_are_clipped {
                self.mark_sprite_cache_clipped(i);
            }

            // Fetch this from the sprite cache
            let (cached_sprite_area, cached_sprite) = match self.sprite_cache_entry(i) {
                Some(x) => x,
                // If clipped, we don't need to draw this
                None => continue,
            };

            height_timer.borrow_mut().start();

            // Lay out this note, so we can work out height
            // We'll also calculate a result here since we might as well
            let navigator = &mut self.rbop_ctx.nav_path.to_navigator();
            let mut current_layout = None;
            let result = if self.current_calculation_idx == i {
                // If this is the calculation currently being edited, there is a possibly edited
                // version in the rbop context, so use that for layout and such
                layout_timer.borrow_mut().start();
                let layout = framework().layout(&self.rbop_ctx.root, Some(navigator), LayoutComputationProperties::default());
                layout_timer.borrow_mut().stop();
                eval_timer.borrow_mut().start();
                let result =  match self.rbop_ctx.root.upgrade() {
                    Ok(structured) => match structured.evaluate() {
                        Ok(evaluation_result) => CalculationResult::Ok(evaluation_result),
                        Err(err) => CalculationResult::MathsError(err),
                    },
                    Err(err) => CalculationResult::NodeError(err),
                };
                eval_timer.borrow_mut().stop();

                current_layout = Some(layout);

                result
            } else {
                self.calculations[i].result.clone()
            };

            // Work out Y position to draw everything from. Since we draw from bottom to top, we
            // need to subtract the height of what we're drawing from base Y
            let node_height = if let Some(ref l) = current_layout {
                l.area.height
            } else {
                cached_sprite_area.height
            };
            let result_height = self.result_height(&result);
            area_timer.borrow_mut().start();
            let this_calculation_lowest_y =
                // Global start
                next_calculation_highest_y - (
                    // Node
                    node_height + PADDING +
                    // Result
                    result_height
                ) as i64;
            area_timer.borrow_mut().stop();

            // If the lowest Y is off the top of the screen (it could still be partially visible)...
            if this_calculation_lowest_y < OperatingSystemInterface::TITLE_BAR_HEIGHT {
                // Then everything else is off the screen
                rest_are_clipped = true;

                // Is this the current calculation? If so, we've scrolled up but this was off the
                // screen, and we need to adjust the starting Y to show this entire calculation 
                if self.current_calculation_idx == i {
                    // TODO: this does a weirdly large scroll if the calculation at the top of the
                    // screen gets taller

                    // The amount which is off the screen (not including the title bar) happens to
                    // be abs(this_calculation_lowest_y), so we can scroll by that amount by adding
                    // it to the starting Y...
                    self.starting_y += this_calculation_lowest_y.abs();

                    // But we also need to account for that title bar!
                    self.starting_y += OperatingSystemInterface::TITLE_BAR_HEIGHT;

                    // Redraw
                    self.tick();
                    return;
                }
            }

            // If the greatest Y is off the bottom of the screen (again, maybe still partially
            // visible), and this is the calculation we're currently editing...
            if next_calculation_highest_y > framework().display.height as i64
                && self.current_calculation_idx == i
            {
                // We need to scroll down by the different between the height of the display and the
                // highest Y
                self.starting_y -= next_calculation_highest_y - framework().display.height as i64;
                
                // Redraw
                self.tick();
                return;
            }
            
            // The next calculation, drawn above this one, should have its highest Y be the same as
            // the lowest Y of this equation, minus one so they don't overlap
            next_calculation_highest_y = this_calculation_lowest_y - 1;

            height_timer.borrow_mut().stop();
            draw_node_timer.borrow_mut().start();
            
            // Set up rbop location
            framework().rbop_location_x = PADDING as i64;
            framework().rbop_location_y = this_calculation_lowest_y + PADDING as i64;
            
            // Is this item being edited?
            if self.current_calculation_idx == i {
                // Draw active nodes
                framework().draw_all_by_layout(
                    &current_layout.unwrap(),
                    self.rbop_ctx.viewport.as_ref(),
                );
            } else {
                // Draw stored nodes
                framework().display.draw_sprite(
                    framework().rbop_location_x,
                    framework().rbop_location_y,
                    &cached_sprite,
                )
            }

            // As we draw different components of the calculation, we'll add to the current Y
            // accordingly.
            let mut this_calculation_current_y = this_calculation_lowest_y;
            this_calculation_current_y += (node_height + PADDING) as i64;

            draw_node_timer.borrow_mut().stop();
            draw_result_timer.borrow_mut().start();
            
            // Draw result
            self.draw_result(this_calculation_current_y, &result);
            this_calculation_current_y += result_height as i64;

            // Draw a big line, unless this is the last item
            if i != calculation_count - 1 {
                framework().display.draw_line(
                    0, this_calculation_current_y as i64,
                    framework().display.width as i64, this_calculation_current_y as i64,
                    Colour::WHITE,
                )
            }

            draw_result_timer.borrow_mut().stop();
        }

        // Write title
        os().ui_draw_title("Calculator");

        // Show timings
        if self.show_timing {
            top_level_timer.stop();
            framework().display.print_at(0, 35, &format!("{}", top_level_timer));
        }

        // Push to screen
        framework().display.draw();

        // Poll for input
        if let Some(input) = framework().buttons.wait_press() {
            if input == OSInput::Exe {
                // Save whatever we're editing
                self.save_current();

                // Add a new calculation to the end of the list
                self.calculations.push(Calculation::blank());

                // Move to it
                self.current_calculation_idx = self.calculations.len() - 1;
                self.reset_scroll();

                // Reset the rbop context, and save the new calculation
                self.load_current();  
                self.save_current();

                // Clear the sprite cache
                self.clear_sprite_cache();
            } else if input == OSInput::List {
                match os().ui_open_menu(&["Toggle timing stats".into(), "Clear history".into()], true) {
                    Some(0) => self.show_timing = !self.show_timing,
                    Some(1) => {
                        // Delete from storage
                        os().filesystem.calculations.table.clear(false);
                        
                        // There are too many things to reload manually, just restart the app
                        os().restart_application();
                    }
                    Some(_) => unreachable!(),
                    None => (),
                }
            } else {
                let move_result = self.rbop_ctx.input(input);
                // Move calculations if needed
                if let Some((dir, MoveResult::MovedOut)) = move_result {
                    match dir {
                        MoveVerticalDirection::Up => if self.current_calculation_idx != 0 {
                            self.save_current();
                            self.current_calculation_idx -= 1;
                            self.load_current();
                            self.clear_sprite_cache();
                        },
                        MoveVerticalDirection::Down => if self.current_calculation_idx != self.calculations.len() - 1 {
                            self.save_current();
                            self.current_calculation_idx += 1;
                            self.load_current();
                            self.clear_sprite_cache();
                        },
                    }
                }
            }
        }
    }

    fn destroy(&mut self) {
        self.clear_sprite_cache();
    }
}

impl CalculatorApplication {
    /// Completely clears the sprite cache and frees any allocated sprites. All sprite cache slots
    /// become `Blank` after this.
    fn clear_sprite_cache(&mut self) {
        // Free the sprite cache
        for item in &self.sprite_cache {
            if let &SpriteCacheEntry::Entry { sprite, .. } = item {
                framework().display.free_sprite(sprite);
            }
        }

        // Fill with "Blank"
        self.sprite_cache = Vec::with_capacity(self.calculations.len());
        for _ in 0..(self.calculations.len()) {
            self.sprite_cache.push(SpriteCacheEntry::Blank);
        }
    }

    /// Retrieves an index in the sprite cache, or computes it if the entry is blank. Returns the
    /// area and sprite pointer if the sprite is has not been marked as clipped, otherwise returns
    /// None.
    fn sprite_cache_entry(&mut self, index: usize) -> Option<(Area, Sprite)> {
        if self.sprite_cache[index] == SpriteCacheEntry::Blank {
            // This entry does not exist
            // Grab calculation
            let root = &self.calculations[index].root;

            // Compute layout
            let layout = framework().layout(root, None, LayoutComputationProperties::default());

            // Draw layout onto a new sprite
            let sprite = framework().display.new_sprite(
                layout.area.width as u16,

                // This was off-by-one after switching to my own ILI9341 library
                // No idea why!!
                layout.area.height as u16 + 1
            );
            framework().display.switch_to_sprite(&sprite);
            framework().rbop_location_x = 0;
            framework().rbop_location_y = 0;
            framework().draw_all_by_layout(&layout, None);
            framework().display.switch_to_screen();

            self.sprite_cache[index] = SpriteCacheEntry::Entry {
                area: layout.area,
                sprite
            }
        }

        match self.sprite_cache[index] {
            SpriteCacheEntry::Entry { area, sprite } =>
                Some((area, sprite)),
            SpriteCacheEntry::Clipped => None,
            SpriteCacheEntry::Blank => panic!("sprite cache miss"),
        }
    }

    /// Marks an entry in the sprite cache as being clipped off the screen. Until the sprite cache
    /// is cleared, any calls to `sprite_cache_entry` will return None so that the application loop
    /// can skip drawing off-screen calculations.
    fn mark_sprite_cache_clipped(&mut self, index: usize) {
        self.sprite_cache[index] = SpriteCacheEntry::Clipped;
    }

    fn save_current(&mut self) {
        // Evaluate
        let result = match self.rbop_ctx.root.upgrade() {
            Ok(structured) => match structured.evaluate() {
                Ok(evaluation_result) => CalculationResult::Ok(evaluation_result),
                Err(err) => CalculationResult::MathsError(err),
            },
            Err(err) => CalculationResult::NodeError(err),
        };

        // Save into array
        self.calculations[self.current_calculation_idx].root = self.rbop_ctx.root.clone();
        self.calculations[self.current_calculation_idx].result = result;

        // Save to storage
        os().filesystem.calculations.write_calculation_at_index(
            ChunkIndex(self.current_calculation_idx as u16),
            self.calculations[self.current_calculation_idx].clone()
        );
    }
    
    fn load_current(&mut self) {
        // Reset rbop context
        self.rbop_ctx = RbopContext {
            viewport: Some(Viewport::new(Area::new(
                framework().display.width - PADDING * 2,
                framework().display.height - PADDING * 2,
            ))),
            root: self.calculations[self.current_calculation_idx].root.clone(),
            ..RbopContext::new()
        };
    }
    

    fn result_height(&self, result: &CalculationResult) -> u64 {
        // If there isn't a result, just imagine that there's a decimal
        let number = if let CalculationResult::Ok(result) = result {
            result.clone()
        } else {
            Number::Decimal(Decimal::zero())
        };

        // Convert the result number into a structured node
        let result_node = StructuredNode::Number(number.clone());

        // Compute a layout for it, so that we know its width and can therefore right-align it
        let result_layout = framework().layout(&result_node, None, LayoutComputationProperties::default());

        // Return the height it will be when drawn, plus padding
        PADDING * 3 + result_layout.area.height
    }

    fn draw_result(&self, y: i64, result: &CalculationResult) {
        // Draw a line
        framework().display.draw_line(
            PADDING as i64, y + PADDING as i64,
            (framework().display.width - PADDING) as i64, y + PADDING as i64,
            Colour::GREY
        );

        let error_string;

        match result {
            CalculationResult::Ok(number) => {
                // Convert the result number into a structured node
                let result_node = StructuredNode::Number(number.clone());

                // Compute a layout for it, so that we know its width and can therefore right-align it
                let result_layout = framework().layout(&result_node, None, LayoutComputationProperties::default());

                // Set up layout location
                framework().rbop_location_x = ((framework().display.width - PADDING) - result_layout.area.width) as i64;
                framework().rbop_location_y = y + PADDING as i64 * 2;

                // Draw
                framework().draw_all_by_layout(&result_layout, None);

                // Don't print an error string
                return;
            },

            CalculationResult::MathsError(err) => error_string = format!("{}", err),
            CalculationResult::NodeError(err) => error_string = format!("{}", err),

            CalculationResult::None => return,
        }

        // That `match` didn't return, print an error string
        let (error_string_width, _) = framework().display.string_size(&error_string);

        let x = (framework().display.width - PADDING) as i64 - error_string_width;
        let y = y + PADDING as i64 * 2;

        framework().display.print_at(x, y, &error_string);
    }

    fn reset_scroll(&mut self) {
        self.starting_y = framework().display.height as i64;
    }
}

