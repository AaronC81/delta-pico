use alloc::{vec::Vec, string::ToString};
use rbop::{Number, StructuredNode, node::{unstructured::{Upgradable, UnstructuredNodeRoot}, structured::EvaluationSettings}, error::{MathsError, NodeError}};
use rust_decimal::prelude::{One, ToPrimitive, Zero};

use crate::{interface::{Colour, ApplicationFramework, ButtonInput, DISPLAY_WIDTH, DISPLAY_HEIGHT}, operating_system::{OSInput, OperatingSystem, os_accessor, OperatingSystemPointer}};
use super::{Application, ApplicationInfo};

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
        DISPLAY_HEIGHT as i16 / 2 - y.to_decimal().to_i16().unwrap()
    }
}

struct Plot {
    root: UnstructuredNodeRoot,
    y_values: Vec<Result<Number, MathsError>>,
}

impl Plot {
    fn recalculate_values(&mut self, view: &ViewWindow, settings: &EvaluationSettings) -> Result<(), NodeError> {
        let upgraded = self.root.upgrade()?;
        self.y_values = view.x_coords_on_screen()
            .iter().map(|i| Self::recalculate_one_value(*i, &upgraded, settings)).collect::<Vec<_>>();

        Ok(())
    }

    fn recalculate_one_value(x: Number, node: &StructuredNode, settings: &EvaluationSettings) -> Result<Number, MathsError> {
        let sn_clone = node.substitute_variable(
            'x',
            &StructuredNode::Number(x)
        );
        sn_clone.evaluate(settings)
    }

    fn recalculate_x_pan(&mut self, pan: isize, view: &ViewWindow, settings: &EvaluationSettings) -> Result<(), NodeError> {
        let upgraded = self.root.upgrade()?;
        let x_values = view.x_coords_on_screen();

        if pan > 0 {
            // Moving right - copy values down
            let pan = pan as usize;
            for i in 0..(self.y_values.len() - pan) {
                self.y_values[i] = self.y_values[i + pan].clone();
            }

            // Insert new values
            for i in (self.y_values.len() - pan)..self.y_values.len() {
                self.y_values[i] = Self::recalculate_one_value(x_values[i], &upgraded, settings);
            }
        } else if pan < 0 {
            // Moving left - copy values up
            let pan = pan.abs() as usize;
            for i in (0..(self.y_values.len() - pan)).rev() {
                self.y_values[i + pan] = self.y_values[i].clone();
            }
            
            // Insert new values
            for i in 0..pan {
                self.y_values[i] = Self::recalculate_one_value(x_values[i], &upgraded, settings);
            }
        }

        Ok(())
    }
}

pub struct GraphApplication<F: ApplicationFramework + 'static> {
    os: OperatingSystemPointer<F>,
    plots: Vec<Plot>,
    view_window: ViewWindow,
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

    fn new(os: OperatingSystemPointer<F>) -> Self {
        Self {
            os,
            plots: Vec::new(),
            view_window: ViewWindow::new(),
        }
    }

    fn tick(&mut self) {
        self.draw();

        // Poll for input
        if let Some(input) = self.os_mut().input() {
            let pan_amount = Number::from(Self::PAN_AMOUNT as i64);
            match input {
                OSInput::Button(ButtonInput::MoveLeft) => {
                    self.view_window.pan_x += pan_amount;
                    let settings = self.settings();
                    for plot in &mut self.plots {
                        plot.recalculate_x_pan(-Self::PAN_AMOUNT, &self.view_window, &settings).expect("unchanged plot encountered error");
                    }
                },
                OSInput::Button(ButtonInput::MoveRight) => {
                    self.view_window.pan_x -= pan_amount;
                    let settings = self.settings();
                    for plot in &mut self.plots {
                        plot.recalculate_x_pan(Self::PAN_AMOUNT, &self.view_window, &settings).expect("unchanged plot encountered error");
                    }
                }
                OSInput::Button(ButtonInput::MoveUp) => {
                    self.view_window.pan_y -= pan_amount;
                }
                OSInput::Button(ButtonInput::MoveDown) => {
                    self.view_window.pan_y += pan_amount;
                }

                OSInput::Button(ButtonInput::List) => self.open_menu(),

                _ => (),
            }
        }
    }
}

impl<F: ApplicationFramework> GraphApplication<F> {
    const PAN_AMOUNT: isize = 10;

    fn draw(&mut self) {
        self.os_mut().display_sprite.fill(Colour::BLACK);

        // Draw axes
        let (x_axis, y_axis) = self.view_window.axis_screen_coords();
        self.os_mut().display_sprite.draw_line(x_axis, 0, x_axis, DISPLAY_HEIGHT as i16, Colour::BLUE);
        self.os_mut().display_sprite.draw_line(0, y_axis, DISPLAY_WIDTH as i16, y_axis, Colour::BLUE);

        // Upgrade, substitute, and draw each graph
        for plot in &self.plots {
            let mut next_y_screen = self.view_window.y_to_screen(plot.y_values[0].clone().unwrap_or(Number::zero()));
            for this_x in 0..(plot.y_values.len() - 1) {
                let next_x = this_x + 1;

                plot.y_values[this_x].as_ref().unwrap();
                if let Ok(this_y) = plot.y_values[this_x] {
                    let next_y = plot.y_values[next_x].as_ref().unwrap_or(&this_y);
                    let this_y_screen = next_y_screen;
                    next_y_screen = self.view_window.y_to_screen(*next_y);
        
                    self.os_mut().display_sprite.draw_line(
                        this_x as i16, this_y_screen,
                        next_x as i16, next_y_screen,
                        Colour::WHITE
                    );
                }
            }
        }

        // Push to screen
        self.os_mut().draw();
    }

    fn open_menu(&mut self) {
        let idx = self.os_mut().ui_open_menu(&[
            "Add plot".into(),
            "View window".into(),
        ], true);
        self.draw();

        match idx {
            Some(0) => {
                // Take input repeatedly until we get something which calculates valid values
                let mut root = None;
                let plot = loop {
                    root = Some(self.os_mut().ui_input_expression("y =", root));
                    let mut plot = Plot { root: root.clone().unwrap(), y_values: Vec::new() };
                    match plot.recalculate_values(&self.view_window, &self.settings()) {
                        Ok(_) => break plot,
                        Err(e) => {
                            self.os_mut().ui_text_dialog(&e.to_string());
                            self.draw();
                        }
                    }
                };
                self.plots.push(plot);
            }

            Some(1) => {
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

                        let settings = self.settings();
                        for plot in &mut self.plots {
                            plot.recalculate_values(&self.view_window, &settings).expect("error rescaling plot");
                        }
                    }
                    Some(1) => {
                        self.view_window.scale_y = self.os_mut().ui_input_expression_and_evaluate(
                            "Y scale:",
                            None,
                            || (),
                        );

                        let settings = self.settings();
                        for plot in &mut self.plots {
                            plot.recalculate_values(&self.view_window, &settings).expect("error rescaling plot");
                        }
                    }
                    None => (),
                    _ => unreachable!()
                }
            },
            None => (),
            _ => unreachable!()
        }
    }

    fn settings(&self) -> EvaluationSettings {
        EvaluationSettings {
            use_floats: true,
            ..self.os().filesystem.settings.evaluation_settings()
        }
    }
}
