use alloc::{string::{String, ToString}, vec};
use rbop::{UnstructuredNodeList, nav::NavPath, node::unstructured::{UnstructuredNodeRoot, Upgradable}, render::{Area, Renderer, Viewport}};

use crate::rbop_impl::{RbopContext, PADDING};
use super::Application;
use crate::interface::framework;

pub struct CalculatorApplication {
    rbop_ctx: RbopContext,
}

impl Application for CalculatorApplication {
    fn name() -> String { "Calculator".into() }
    fn visible() -> bool { true }

    fn new() -> Self {
        Self {
            rbop_ctx: RbopContext {
                root: UnstructuredNodeRoot { root: UnstructuredNodeList { items: vec![] } },
                nav_path: NavPath::new(vec![0]),
                viewport: Some(Viewport::new(Area::new(
                    framework().display.width - PADDING * 2,
                    framework().display.height - PADDING * 2,
                ))),
            }
        }
    }

    fn tick(&mut self) {
        // Draw
        framework().draw_all(
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

        // Write result
        if let Some(result) = result {
            let result_str = result.to_string();
            let mut result_chars = result_str.as_bytes().to_vec();
            result_chars.push(0);

            (framework().display.set_cursor)(
                0,
                (framework().display.height - PADDING * 2 - 30) as i64
            );
            (framework().display.print)(result_chars.as_ptr());
        }

        // Push to screen
        (framework().display.draw)();

        // Poll for input
        if let Some(input) = framework().buttons.poll_press() {
            self.rbop_ctx.input(input);
        }
    }
}
