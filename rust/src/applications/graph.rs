use alloc::{format, string::{String, ToString}, vec::{self, Vec}};
use rbop::{StructuredNode, UnstructuredNodeList, nav::NavPath, node::unstructured::{UnstructuredNodeRoot, Upgradable}, render::{Area, Renderer, Viewport}};
use rust_decimal::{Decimal, prelude::{FromPrimitive, ToPrimitive}};

use crate::{interface::ButtonInput, operating_system::os, rbop_impl::{RbopContext, PADDING}};
use super::{Application, ApplicationInfo};
use crate::interface::framework;
use crate::graphics::colour;

pub struct ViewWindow {
    pan_x: Decimal,
    pan_y: Decimal,
    scale_x: Decimal,
    scale_y: Decimal,
}

impl ViewWindow {
    fn new() -> ViewWindow {
        ViewWindow {
            pan_x: Decimal::ZERO,
            pan_y: Decimal::ZERO,
            scale_x: Decimal::ONE,
            scale_y: Decimal::ONE,
        }
    }

    fn axis_screen_coords(&self) -> (i64, i64) {
        (
            self.x_to_screen(Decimal::ZERO),
            self.y_to_screen(Decimal::ZERO)
        )
    }

    /// Returns the X values which can currently be seen on the screen. The
    /// vector contains one value per X pixel; to calculate all necessary graph
    /// values, iterate over these.
    fn x_coords_on_screen(&self) -> Vec<Decimal> {
        // Delta between X pixels is 1 / scale
        let x_delta = Decimal::ONE / self.scale_x;

        let x_start = (Decimal::from_i64(
            framework().display.width as i64 / -2
        ).unwrap() - self.pan_x) * x_delta;

        (0..framework().display.width)
            .map(|i| x_start + Decimal::from_u64(i).unwrap() * x_delta)
            .collect::<Vec<_>>()
    }

    /// Given a X value in the graph space, returns a X value on the screen.
    fn x_to_screen(&self, mut x: Decimal) -> i64 {
        // Apply scale
        x *= self.scale_x;

        // Apply user-specified pan
        x += self.pan_x;

        // Squash into an integer, and pan so that (0, 0) is in the middle of
        // the screen
        x.to_i64().unwrap() + framework().display.width as i64 / 2
    }

    /// Given a Y value in the graph space, returns a Y value on the screen.
    fn y_to_screen(&self, mut y: Decimal) -> i64 {
        // Apply scale
        y *= self.scale_y;

        // Apply user-specified pan
        y += self.pan_y;

        // Squash into an integer, flip around the bottom of the screen, and
        // pan so that (0, 0) is in the middle of the screen
        (framework().display.height as i64 + -1 * y.to_i64().unwrap())
            - framework().display.height as i64 / 2
    }
}

pub struct GraphApplication {
    rbop_ctx: RbopContext,
    view_window: ViewWindow,
}

impl Application for GraphApplication {
    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "Graph".into(),
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
            view_window: ViewWindow::new(),
        }
    }

    fn tick(&mut self) {
        (framework().display.fill_screen)(colour::BLACK);
        
        // Draw rbop input
        framework().draw_all(
            &self.rbop_ctx.root, 
            Some(&mut self.rbop_ctx.nav_path.to_navigator()),
            self.rbop_ctx.viewport.as_ref(),
        );

        // Draw axes
        let (x_axis, y_axis) = self.view_window.axis_screen_coords();
        (framework().display.draw_line)(x_axis, 0, x_axis, framework().display.height as i64, colour::BLUE);
        (framework().display.draw_line)(0, y_axis, framework().display.width as i64, y_axis, colour::BLUE);

        // Upgrade, substitute, and draw graph
        if let Ok(sn) = self.rbop_ctx.root.upgrade() {
            let func = |x| {
                let sn_clone = sn.substitute_variable(
                    'x',
                    &StructuredNode::Number(x)
                );
                sn_clone.evaluate()
            };
            let values = self.view_window.x_coords_on_screen()
                .iter().map(|i| func(*i)).collect::<Vec<_>>();
    
            for this_x in 0..(values.len() - 1) {
                let next_x = this_x + 1;

                values[this_x].as_ref().unwrap();
                if let Ok(this_y) = values[this_x] {
                    let next_y = values[next_x].as_ref().unwrap_or(&this_y);
        
                    (framework().display.draw_line)(
                        this_x as i64, self.view_window.y_to_screen(this_y),
                        next_x as i64, self.view_window.y_to_screen(*next_y),
                        colour::WHITE
                    );
                }
            }
        }

        // Push to screen
        (framework().display.draw)();

        // Poll for input
        if let Some(input) = framework().buttons.poll_press() {
            self.rbop_ctx.input(input);
        }
    }
}
