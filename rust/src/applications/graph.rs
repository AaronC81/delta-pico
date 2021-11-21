use alloc::{format, string::{String, ToString}, vec, vec::{Vec}};
use rbop::{Number, StructuredNode, UnstructuredNodeList, nav::NavPath, node::unstructured::{UnstructuredNodeRoot, Upgradable}, render::{Area, Renderer, Viewport}};
use rust_decimal::prelude::{One, ToPrimitive, Zero};

use crate::{interface::{ButtonInput, Colour}, operating_system::{OSInput, os}, rbop_impl::{RbopContext}};
use super::{Application, ApplicationInfo};
use crate::interface::framework;

const PADDING: u64 = 10;

pub struct ViewWindow {
    pan_x: Number,
    pan_y: Number,
    scale_x: Number,
    scale_y: Number,
}

impl ViewWindow {
    fn new() -> ViewWindow {
        ViewWindow {
            pan_x: Number::zero(),
            pan_y: Number::zero(),
            scale_x: Number::one(),
            scale_y: Number::one(),
        }
    }

    fn axis_screen_coords(&self) -> (i64, i64) {
        (
            self.x_to_screen(Number::zero()),
            self.y_to_screen(Number::zero())
        )
    }

    /// Returns the X values which can currently be seen on the screen. The
    /// vector contains one value per X pixel; to calculate all necessary graph
    /// values, iterate over these.
    fn x_coords_on_screen(&self) -> Vec<Number> {
        // Delta between X pixels is 1 / scale
        let x_delta = Number::one() / self.scale_x;

        let x_start = (Number::from(
            framework().display.width as i64 / -2
        ) - self.pan_x) * x_delta;

        (0..framework().display.width)
            .map(|i| x_start + Number::from(i as i64) * x_delta)
            .collect::<Vec<_>>()
    }

    /// Given a X value in the graph space, returns a X value on the screen.
    fn x_to_screen(&self, mut x: Number) -> i64 {
        // Apply scale
        x *= self.scale_x;

        // Apply user-specified pan
        x += self.pan_x;

        // Squash into an integer, and pan so that (0, 0) is in the middle of
        // the screen
        x.to_decimal().to_i64().unwrap() + framework().display.width as i64 / 2
    }

    /// Given a Y value in the graph space, returns a Y value on the screen.
    fn y_to_screen(&self, mut y: Number) -> i64 {
        // Apply scale
        y *= self.scale_y;

        // Apply user-specified pan
        y += self.pan_y;

        // Squash into an integer, flip around the bottom of the screen, and
        // pan so that (0, 0) is in the middle of the screen
        (framework().display.height as i64 + -1 * y.to_decimal().to_i64().unwrap())
            - framework().display.height as i64 / 2
    }
}

pub struct GraphApplication {
    rbop_ctx: RbopContext,
    view_window: ViewWindow,
    edit_mode: bool,
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
                    framework().display.width - PADDING * 2 - 30,
                    framework().display.height - PADDING * 2,
                ))),
                ..RbopContext::new()
            },
            view_window: ViewWindow::new(),
            edit_mode: true,
        }
    }

    fn tick(&mut self) {
        self.draw();

        // Poll for input
        if let Some(input) = framework().buttons.wait_press() {
            if input == OSInput::Exe {
                self.edit_mode = !self.edit_mode;        
            } else if self.edit_mode {
                self.rbop_ctx.input(input);
            } else {
                let ten = Number::from(10);
                match input {
                    OSInput::MoveLeft => self.view_window.pan_x += ten,
                    OSInput::MoveRight => self.view_window.pan_x -= ten,
                    OSInput::MoveUp => self.view_window.pan_y -= ten,
                    OSInput::MoveDown => self.view_window.pan_y += ten,

                    OSInput::List => self.open_menu(),

                    _ => (),
                }
            }
        }
    }
}

impl GraphApplication {
    fn draw(&mut self) {
        framework().display.fill_screen(Colour::BLACK);

        if self.edit_mode {
            os().ui_draw_title("Graph");

            // Draw rbop input
            framework().rbop_location_x = PADDING as i64 + 30;
            framework().rbop_location_y = PADDING as i64 + 30;
            let block = framework().draw_all(
                &self.rbop_ctx.root, 
                Some(&mut self.rbop_ctx.nav_path.to_navigator()),
                self.rbop_ctx.viewport.as_ref(),
            );

            // Draw "y="
            (framework().display.set_cursor)(PADDING as i64, PADDING as i64 + 30 + block.baseline as i64 - 8);
            framework().display.print("y=");

            (framework().display.set_cursor)(23, 290);
            framework().display.print("EXE: Toggle edit/view");
        } else {
            // Draw axes
            let (x_axis, y_axis) = self.view_window.axis_screen_coords();
            framework().display.draw_line(x_axis, 0, x_axis, framework().display.height as i64, Colour::BLUE);
            framework().display.draw_line(0, y_axis, framework().display.width as i64, y_axis, Colour::BLUE);

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
            
                        framework().display.draw_line(
                            this_x as i64, self.view_window.y_to_screen(this_y),
                            next_x as i64, self.view_window.y_to_screen(*next_y),
                            Colour::WHITE
                        );
                    }
                }
            }
        }

        // Push to screen
        (framework().display.draw)();
    }

    fn open_menu(&mut self) {
        let idx = os().ui_open_menu(&vec![
            "View window".into(),
        ], true);
        self.draw();

        match idx {
            Some(0) => {
                let idx = os().ui_open_menu(&vec![
                    "X scale".into(),
                    "Y scale".into(),
                ], true);

                match idx {
                    Some(0) => {
                        self.view_window.scale_x = os().ui_input_expression_and_evaluate(
                            "X scale:",
                            None,
                            || self.draw(),
                        );
                    }
                    Some(1) => {
                        self.view_window.scale_y = os().ui_input_expression_and_evaluate(
                            "Y scale:",
                            None,
                            || self.draw(),
                        );
                    }
                    None => (),
                    _ => unreachable!()
                }
            },
            None => (),
            _ => unreachable!()
        }
    }
}
