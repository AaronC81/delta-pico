use alloc::{string::{String, ToString}, vec};
use rbop::{UnstructuredNodeList, nav::NavPath, node::unstructured::{UnstructuredNodeRoot, Upgradable}, render::{Area, CalculatedPoint, Renderer, Viewport}};

use crate::{graphics::colour, rbop_impl::{RbopContext}};
use super::{Application, ApplicationInfo};
use crate::interface::framework;

const PADDING: u64 = 10;

pub struct CalculatorApplication {
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
            }
        }
    }

    fn tick(&mut self) {
        // Clear screen
        (framework().display.fill_screen)(colour::BLACK);
        
        // Draw
        framework().rbop_location_x = PADDING;
        framework().rbop_location_y = PADDING;
        let layout = framework().draw_all(
            &self.rbop_ctx.root, 
            Some(&mut self.rbop_ctx.nav_path.to_navigator()),
            self.rbop_ctx.viewport.as_ref(),
        );
        let input_end_y = layout.area(framework()).height + PADDING;
        
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

        // Draw a line
        (framework().display.draw_line)(
            PADDING as i64, (input_end_y + PADDING) as i64,
            (framework().display.width - PADDING) as i64, (input_end_y + PADDING) as i64,
            colour::GREY
        );

        // Write result
        if let Some(result) = result {
            // Convert decimal to string and truncate
            let mut result_str = result.to_string();
            if result_str.len() > 15 {
                result_str = result_str[0..15].to_string();
            }

            // Calculate length for right-alignment
            let (result_str_len, _) = framework().display.string_size(&result_str);

            // Write text
            (framework().display.set_cursor)(
                (framework().display.width - PADDING) as i64 - result_str_len,
                (input_end_y + PADDING * 2) as i64
            );
            framework().display.print(result_str);
        }

        // Push to screen
        (framework().display.draw)();

        // Poll for input
        if let Some(input) = framework().buttons.poll_press() {
            self.rbop_ctx.input(input);
        }
    }
}
