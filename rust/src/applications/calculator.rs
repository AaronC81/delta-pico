use alloc::{string::{String, ToString}, vec, vec::{Vec}};
use rbop::{Token, UnstructuredNode, UnstructuredNodeList, nav::{MoveVerticalDirection, NavPath}, node::unstructured::{MoveResult, UnstructuredNodeRoot, Upgradable}, render::{Area, CalculatedPoint, Renderer, Viewport}};
use rust_decimal::Decimal;

use crate::{filesystem::{Calculation, ChunkIndex}, graphics::colour, interface::ButtonInput, operating_system::os, rbop_impl::{RbopContext}};
use super::{Application, ApplicationInfo};
use crate::interface::framework;

const PADDING: u64 = 10;

pub struct CalculatorApplication {
    calculations: Vec<Calculation>,
    current_calculation_idx: usize,
    rbop_ctx: RbopContext,
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
        Self {
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
        }
    }

    fn tick(&mut self) {
        // TODO: assumes that all calculations fit on screen, which will not be the case

        // Clear screen
        (framework().display.fill_screen)(colour::BLACK);
        os().ui_draw_title("Calculator");

        let mut calc_start_y = 30_u64;
        
        // Draw history
        // TODO: clone is undoubtedly very inefficient here, but it makes the borrow checker happy
        let items = self.calculations.iter().cloned().enumerate().collect::<Vec<_>>();
        for (i, Calculation { root, result }) in &items {
            // Set up rbop location
            framework().rbop_location_x = PADDING;
            framework().rbop_location_y = calc_start_y + PADDING;
            
            // Is this item being edited?
            let (layout, result) = if self.current_calculation_idx == *i {
                // Draw active nodes
                let layout = framework().draw_all(
                    &self.rbop_ctx.root, 
                    Some(&mut self.rbop_ctx.nav_path.to_navigator()),
                    self.rbop_ctx.viewport.as_ref(),
                );

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

                (layout, result)
            } else {
                // Draw stored nodes
                let layout = framework().draw_all(
                    root, None, None,
                );

                (layout, result.clone())
            };

            calc_start_y += layout.area(framework()).height + PADDING;
            
            // Draw result
            calc_start_y += self.draw_result(calc_start_y, result.clone());

            // Draw a big line, unless this is the last item
            if i != &(items.len() - 1) {
                (framework().display.draw_line)(
                    0, calc_start_y as i64,
                    framework().display.width as i64, calc_start_y as i64,
                    colour::WHITE,
                )
            }
        }

        // Push to screen
        (framework().display.draw)();

        // Poll for input
        if let Some(input) = framework().buttons.poll_press() {
            if input == ButtonInput::Exe {
                self.save_current();
                self.calculations.push(Calculation::blank());
                self.current_calculation_idx += 1;
                self.load_current();  
                self.save_current();
            } else {
                let move_result = self.rbop_ctx.input(input);
                // Move calculations if needed
                if let Some((dir, MoveResult::MovedOut)) = move_result {
                    match dir {
                        MoveVerticalDirection::Up => if self.current_calculation_idx != 0 {
                            self.save_current();
                            self.current_calculation_idx -= 1;
                            self.load_current();
                        },
                        MoveVerticalDirection::Down => if self.current_calculation_idx != self.calculations.len() - 1 {
                            self.save_current();
                            self.current_calculation_idx += 1;
                            self.load_current();
                        },
                    }
                }
            }
        }
    }
}

impl CalculatorApplication {
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

    fn draw_result(&mut self, y: u64, result: Option<Decimal>) -> u64 {
        // Draw a line
        (framework().display.draw_line)(
            PADDING as i64, (y + PADDING) as i64,
            (framework().display.width - PADDING) as i64, (y + PADDING) as i64,
            colour::GREY
        );

        // Write result
        let mut result_str_height = 0;
        if let Some(result) = result {
            // Convert decimal to string and truncate
            let mut result_str = result.to_string();
            if result_str.len() > 15 {
                result_str = result_str[0..15].to_string();
            }
            
            // Calculate length for right-alignment
            let (result_str_len, h) = framework().display.string_size(&result_str);
            result_str_height = h;

            // Write text
            (framework().display.set_cursor)(
                (framework().display.width - PADDING) as i64 - result_str_len,
                (y + PADDING * 2) as i64
            );
            framework().display.print(result_str);
        }

        PADDING * 3 + result_str_height as u64
    }
}
