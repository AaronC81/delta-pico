use alloc::{format, string::{String, ToString}, vec, vec::{Vec}};
use rbop::{Token, UnstructuredNode, UnstructuredNodeList, nav::{MoveVerticalDirection, NavPath}, node::{self, unstructured::{MoveResult, UnstructuredNodeRoot, Upgradable}}, render::{Area, CalculatedPoint, Layoutable, Renderer, Viewport}};
use rust_decimal::Decimal;

use crate::{filesystem::{Calculation, ChunkIndex}, graphics::colour, interface::ButtonInput, operating_system::os, rbop_impl::{RbopContext}, timer::Timer};
use super::{Application, ApplicationInfo};
use crate::interface::framework;

const PADDING: u64 = 10;

pub struct CalculatorApplication {
    calculations: Vec<Calculation>,
    current_calculation_idx: usize,
    rbop_ctx: RbopContext,
    sprite_cache: Vec<(Area, *mut u8)>,
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
        };
        result.compute_sprite_cache();
        result
    }

    fn tick(&mut self) {
        // TODO: assumes that all calculations fit on screen, which will not be the case

        // Clear screen
        (framework().display.fill_screen)(colour::BLACK);

        let mut calc_block_start_y = framework().display.height as i64;

        let result_string_height = framework().display.string_size("A").1;

        let mut top_level_timer = Timer::new("Tick");
        top_level_timer.start();
        let height_timer = top_level_timer.new_subtimer("Height");
        let layout_timer = height_timer.borrow_mut().new_subtimer("Layout");
        let eval_timer = height_timer.borrow_mut().new_subtimer("Eval");
        let area_timer = height_timer.borrow_mut().new_subtimer("Area calc");

        let draw_node_timer = top_level_timer.new_subtimer("Draw node");
        let draw_result_timer = top_level_timer.new_subtimer("Draw result");
        
        // Draw history
        // TODO: can prune calculations which are entirely off the screen
        let calculation_count = self.calculations.len();
        let items = self.calculations.iter().enumerate().rev();
        for (i, Calculation { result, .. }) in items {
            height_timer.borrow_mut().start();

            // Lay out this note, so we can work out height
            // We'll also calculate a result here since we might as well
            let navigator = &mut self.rbop_ctx.nav_path.to_navigator();
            let mut current_layout = None;
            let result = if self.current_calculation_idx == i {
                // If this is the calculation currently being edited, there is a possibly edited
                // version in the rbop context, so use that for layout and such
                layout_timer.borrow_mut().start();
                let layout = framework().layout(&self.rbop_ctx.root, Some(navigator));
                layout_timer.borrow_mut().stop();
                eval_timer.borrow_mut().start();
                let result = if let Ok(structured) = self.rbop_ctx.root.upgrade() {
                    if let Ok(evaluation_result) = structured.evaluate() {
                        Some(evaluation_result)
                    } else {
                        None
                    }
                } else {
                    None
                };
                eval_timer.borrow_mut().stop();

                current_layout = Some(layout);

                result
            } else {
                *result
            };

            // Work out Y position to draw everything from
            let node_height = if let Some(ref l) = current_layout {
                l.area.height
            } else {
                self.sprite_cache[i].0.height
            };
            area_timer.borrow_mut().start();
            let mut calc_start_y =
                // Global start
                calc_block_start_y - (
                    // Node
                    node_height + PADDING +
                    // Result
                    PADDING * 3 + result_string_height as u64
                ) as i64;
            area_timer.borrow_mut().stop();
            
            calc_block_start_y = calc_start_y;

            height_timer.borrow_mut().stop();
            draw_node_timer.borrow_mut().start();
            
            // Set up rbop location
            framework().rbop_location_x = PADDING as i64;
            framework().rbop_location_y = calc_start_y + PADDING as i64;
            
            // Is this item being edited?
            if self.current_calculation_idx == i {
                // Draw active nodes
                framework().draw_all_by_layout(
                    &current_layout.unwrap(),
                    self.rbop_ctx.viewport.as_ref(),
                );
            } else {
                // Draw stored nodes
                (framework().display.draw_sprite)(
                    framework().rbop_location_x,
                    framework().rbop_location_y,
                    self.sprite_cache[i].1,
                )
            }

            calc_start_y += (node_height + PADDING) as i64;

            draw_node_timer.borrow_mut().stop();
            draw_result_timer.borrow_mut().start();
            
            // Draw result
            calc_start_y += self.draw_result(calc_start_y, &result) as i64;

            // Draw a big line, unless this is the last item
            if i != calculation_count - 1 {
                (framework().display.draw_line)(
                    0, calc_start_y as i64,
                    framework().display.width as i64, calc_start_y as i64,
                    colour::WHITE,
                )
            }

            draw_result_timer.borrow_mut().stop();
        }

        // Write title
        os().ui_draw_title("Calculator");

        // Show timings
        top_level_timer.stop();
        (framework().display.set_cursor)(0, 35);
        framework().display.print(format!("{}", top_level_timer));

        // Push to screen
        (framework().display.draw)();

        // Poll for input
        if let Some(input) = framework().buttons.wait_press() {
            if input == ButtonInput::Exe {
                self.save_current();
                self.calculations.push(Calculation::blank());
                self.current_calculation_idx += 1;
                self.load_current();  
                self.save_current();
                self.compute_sprite_cache();
            } else {
                let move_result = self.rbop_ctx.input(input);
                // Move calculations if needed
                if let Some((dir, MoveResult::MovedOut)) = move_result {
                    match dir {
                        MoveVerticalDirection::Up => if self.current_calculation_idx != 0 {
                            self.save_current();
                            self.current_calculation_idx -= 1;
                            self.load_current();
                            self.compute_sprite_cache();
                        },
                        MoveVerticalDirection::Down => if self.current_calculation_idx != self.calculations.len() - 1 {
                            self.save_current();
                            self.current_calculation_idx += 1;
                            self.load_current();
                            self.compute_sprite_cache();
                        },
                    }
                }
            }
        }
    }

    fn destroy(&mut self) {
        self.free_sprite_cache();
    }
}

impl CalculatorApplication {
    fn free_sprite_cache(&mut self) {
        // Free the sprite cache
        for (_, s) in &self.sprite_cache {
            (framework().display.free_sprite)(*s);
        }        
    }

    fn compute_sprite_cache(&mut self) {
        self.free_sprite_cache();

        // Create new cache
        self.sprite_cache = self.calculations.iter().cloned()
            .map(|Calculation { root, .. }| {
                // Compute layout
                let layout = framework().layout(&root, None);

                // Draw layout onto a new sprite
                let sprite = (framework().display.new_sprite)(
                    layout.area.width as i16, layout.area.height as i16
                );
                (framework().display.switch_to_sprite)(sprite);
                framework().rbop_location_x = 0;
                framework().rbop_location_y = 0;
                framework().draw_all_by_layout(&layout, None);
                (framework().display.switch_to_screen)();

                (layout.area, sprite)
            })
            .collect::<Vec<_>>();
    }

    fn save_current(&mut self) {
        // Evaluate
        let result = if let Ok(structured) = self.rbop_ctx.root.upgrade() {
            if let Ok(evaluation_result) = structured.evaluate() {
                Some(evaluation_result)
            } else {
                None
            }
        } else {
            None
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

    fn draw_result(&self, y: i64, result: &Option<Decimal>) -> u64 {
        // Draw a line
        (framework().display.draw_line)(
            PADDING as i64, y + PADDING as i64,
            (framework().display.width - PADDING) as i64, y + PADDING as i64,
            colour::GREY
        );

        // Calculate result text
        let result_str = if let Some(result) = result {
            // Convert decimal to string and truncate
            let mut result_str = result.to_string();
            if result_str.len() > 15 {
                result_str = result_str[0..15].to_string();
            }
                
            result_str
        } else {
            " ".into()
        };

        // Calculate length for right-alignment
        let (result_str_len, h) = framework().display.string_size(&result_str);
        let result_str_height = h;

        // Write text
        (framework().display.set_cursor)(
            (framework().display.width - PADDING) as i64 - result_str_len,
            y + PADDING as i64 * 2
        );
        framework().display.print(result_str);

        PADDING * 3 + result_str_height as u64
    }
}

