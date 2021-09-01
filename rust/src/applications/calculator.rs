use alloc::{string::{String, ToString}, vec, vec::{Vec}};
use rbop::{Token, UnstructuredNode, UnstructuredNodeList, nav::NavPath, node::unstructured::{UnstructuredNodeRoot, Upgradable}, render::{Area, CalculatedPoint, Renderer, Viewport}};
use rust_decimal::Decimal;

use crate::{graphics::colour, rbop_impl::{RbopContext}};
use super::{Application, ApplicationInfo};
use crate::interface::framework;

const PADDING: u64 = 10;

pub struct CalculatorApplication {
    calculation_history: Vec<(UnstructuredNodeRoot, Decimal)>,
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
        Self {
            rbop_ctx: RbopContext {
                viewport: Some(Viewport::new(Area::new(
                    framework().display.width - PADDING * 2,
                    framework().display.height - PADDING * 2,
                ))),
                ..RbopContext::new()
            },
            // TODO
            calculation_history: vec![
                (UnstructuredNodeRoot { root: UnstructuredNodeList { items: vec![
                    UnstructuredNode::Token(Token::Digit(1)),
                    UnstructuredNode::Token(Token::Add),
                    UnstructuredNode::Token(Token::Digit(2)),
                ] } }, Decimal::TWO + Decimal::ONE),
                (UnstructuredNodeRoot { root: UnstructuredNodeList { items: vec![
                    UnstructuredNode::Token(Token::Digit(3)),
                    UnstructuredNode::Token(Token::Add),
                    UnstructuredNode::Token(Token::Digit(4)),
                ] } }, Decimal::TWO + Decimal::TWO + Decimal::TWO + Decimal::ONE),
            ],
        }
    }

    fn tick(&mut self) {
        // TODO: assumes that all calculations fit on screen, which will not be the case

        // Clear screen
        (framework().display.fill_screen)(colour::BLACK);

        let mut calc_start_y = 0_u64;
        
        // Draw history
        // TODO: clone is undoubtedly very inefficient here, but it makes the borrow checker happy
        let items = self.calculation_history.iter().cloned().collect::<Vec<_>>();
        for (node, result) in items {
            // Draw nodes
            framework().rbop_location_x = PADDING;
            framework().rbop_location_y = calc_start_y + PADDING;
            let layout = framework().draw_all(
                &node, None, None,
            );
            calc_start_y += layout.area(framework()).height + PADDING;

            // Draw result
            calc_start_y += self.draw_result(calc_start_y, Some(result.clone()));

            // Draw a big line
            (framework().display.draw_line)(
                0, calc_start_y as i64,
                framework().display.width as i64, calc_start_y as i64,
                colour::WHITE,
            )
        }

        // Draw
        framework().rbop_location_x = PADDING;
        framework().rbop_location_y = calc_start_y + PADDING;
        let layout = framework().draw_all(
            &self.rbop_ctx.root, 
            Some(&mut self.rbop_ctx.nav_path.to_navigator()),
            self.rbop_ctx.viewport.as_ref(),
        );
        let input_end_y = framework().rbop_location_y + layout.area(framework()).height;
        
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

        // Draw result
        self.draw_result(input_end_y, result);

        // Push to screen
        (framework().display.draw)();

        // Poll for input
        if let Some(input) = framework().buttons.poll_press() {
            self.rbop_ctx.input(input);
        }
    }
}

impl CalculatorApplication {
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
