use alloc::vec::Vec;
use rbop::{Number, StructuredNode, node::{unstructured::{Upgradable}}, render::{Area, Viewport}};
use rust_decimal::prelude::{One, ToPrimitive, Zero};

use crate::{interface::{Colour, ApplicationFramework, ButtonInput, DISPLAY_WIDTH, DISPLAY_HEIGHT}, operating_system::{OSInput, OperatingSystem, os_accessor}, rbop_impl::{RbopContext, RbopSpriteRenderer}};
use super::{Application, ApplicationInfo};

const PADDING: u16 = 10;

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

    fn axis_screen_coords(&self) -> (i16, i16) {
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
            DISPLAY_WIDTH as i64 / -2
        ) - self.pan_x) * x_delta;

        (0..DISPLAY_WIDTH)
            .map(|i| x_start + Number::from(i as i64) * x_delta)
            .collect::<Vec<_>>()
    }

    /// Given a X value in the graph space, returns a X value on the screen.
    fn x_to_screen(&self, mut x: Number) -> i16 {
        // Apply scale
        x *= self.scale_x;

        // Apply user-specified pan
        x += self.pan_x;

        // Squash into an integer, and pan so that (0, 0) is in the middle of
        // the screen
        x.to_decimal().to_i16().unwrap() + DISPLAY_WIDTH as i16 / 2
    }

    /// Given a Y value in the graph space, returns a Y value on the screen.
    fn y_to_screen(&self, mut y: Number) -> i16 {
        // Apply scale
        y *= self.scale_y;

        // Apply user-specified pan
        y += self.pan_y;

        // Squash into an integer, flip around the bottom of the screen, and
        // pan so that (0, 0) is in the middle of the screen
        (DISPLAY_HEIGHT as i16 + -y.to_decimal().to_i16().unwrap())
            - DISPLAY_HEIGHT as i16 / 2
    }
}

pub struct GraphApplication<F: ApplicationFramework + 'static> {
    os: *mut OperatingSystem<F>,
    rbop_ctx: RbopContext<F>,
    view_window: ViewWindow,
    edit_mode: bool,
}

os_accessor!(GraphApplication<F>);

impl<F: ApplicationFramework> Application for GraphApplication<F> {
    type Framework = F;

    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "Graph".into(),
            visible: true,
        }
    }

    fn new(os: *mut OperatingSystem<F>) -> Self {
        Self {
            os,
            rbop_ctx: RbopContext {
                viewport: Some(Viewport::new(Area::new(
                    (DISPLAY_WIDTH - PADDING * 2 - 30).into(),
                    (DISPLAY_HEIGHT - PADDING * 2).into(),
                ))),
                ..RbopContext::new(os)
            },
            view_window: ViewWindow::new(),
            edit_mode: true,
        }
    }

    fn tick(&mut self) {
        self.draw();

        // Poll for input
        if let Some(input) = self.os_mut().input() {
            if input == OSInput::Button(ButtonInput::Exe) {
                self.edit_mode = !self.edit_mode;        
            } else if self.edit_mode {
                self.rbop_ctx.input(input);
            } else {
                let ten = Number::from(10);
                match input {
                    OSInput::Button(ButtonInput::MoveLeft) => self.view_window.pan_x += ten,
                    OSInput::Button(ButtonInput::MoveRight) => self.view_window.pan_x -= ten,
                    OSInput::Button(ButtonInput::MoveUp) => self.view_window.pan_y -= ten,
                    OSInput::Button(ButtonInput::MoveDown) => self.view_window.pan_y += ten,

                    OSInput::Button(ButtonInput::List) => self.open_menu(),

                    _ => (),
                }
            }
        }
    }
}

impl<F: ApplicationFramework> GraphApplication<F> {
    fn draw(&mut self) {
        self.os_mut().display_sprite.fill(Colour::BLACK);

        if self.edit_mode {
            self.os_mut().ui_draw_title("Graph");

            // Draw rbop input
            let sprite = RbopSpriteRenderer::draw_context_to_sprite(&mut self.rbop_ctx, Colour::BLACK);
            self.os_mut().display_sprite.draw_sprite(PADDING as i16 + 30, PADDING as i16 + 30, &sprite);

            // TODO: Drawing "y=" in line with the baseline would be nice, but we don't actually
            // get given our layout
            self.os_mut().display_sprite.print_at(
                PADDING as i16, PADDING as i16 + 30 - 8,
                "y="
            );

            self.os_mut().display_sprite.print_at(23, 290, "EXE: Toggle edit/view");
        } else {
            // Draw axes
            let (x_axis, y_axis) = self.view_window.axis_screen_coords();
            self.os_mut().display_sprite.draw_line(x_axis, 0, x_axis, DISPLAY_HEIGHT as i16, Colour::BLUE);
            self.os_mut().display_sprite.draw_line(0, y_axis, DISPLAY_WIDTH as i16, y_axis, Colour::BLUE);

            // Upgrade, substitute, and draw graph
            if let Ok(sn) = self.rbop_ctx.root.upgrade() {
                let func = |x| {
                    let sn_clone = sn.substitute_variable(
                        'x',
                        &StructuredNode::Number(x)
                    );
                    sn_clone.evaluate(&self.os().filesystem.settings.evaluation_settings())
                };
                let values = self.view_window.x_coords_on_screen()
                    .iter().map(|i| func(*i)).collect::<Vec<_>>();
        
                for this_x in 0..(values.len() - 1) {
                    let next_x = this_x + 1;

                    values[this_x].as_ref().unwrap();
                    if let Ok(this_y) = values[this_x] {
                        let next_y = values[next_x].as_ref().unwrap_or(&this_y);
            
                        self.os_mut().display_sprite.draw_line(
                            this_x as i16, self.view_window.y_to_screen(this_y),
                            // TODO: use next_x when draw_line supports it
                            this_x as i16, self.view_window.y_to_screen(*next_y),
                            Colour::WHITE
                        );
                    }
                }
            }
        }

        // Push to screen
        self.os_mut().draw();
    }

    fn open_menu(&mut self) {
        let idx = self.os_mut().ui_open_menu(&[
            "View window".into(),
        ], true);
        self.draw();

        match idx {
            Some(0) => {
                let idx = self.os_mut().ui_open_menu(&[
                    "X scale".into(),
                    "Y scale".into(),
                ], true);

                match idx {
                    // TODO: Doesn't redraw because Ferris was angry at me
                    Some(0) => {
                        self.view_window.scale_x = self.os_mut().ui_input_expression_and_evaluate(
                            "X scale:",
                            None,
                            || (),
                        );
                    }
                    Some(1) => {
                        self.view_window.scale_y = self.os_mut().ui_input_expression_and_evaluate(
                            "Y scale:",
                            None,
                            || (),
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
