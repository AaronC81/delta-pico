use alloc::{vec, vec::Vec, string::ToString, format};
use rbop::{Number, StructuredNode, node::{unstructured::{Upgradable, UnstructuredNodeRoot}, structured::EvaluationSettings}, error::MathsError, UnstructuredNodeList, UnstructuredNode, Token};
use rust_decimal::prelude::{One, ToPrimitive, Zero};

use crate::{interface::{Colour, ApplicationFramework, ButtonInput, DISPLAY_WIDTH, DISPLAY_HEIGHT}, operating_system::{OSInput, OperatingSystem, os_accessor, OperatingSystemPointer, ContextMenu, ContextMenuItem, SelectorMenuCallable}};
use super::{Application, ApplicationInfo};

/// Represents the current viewport position and scale.
pub struct ViewWindow {
    /// The X offset of the viewport in pixels, where 0 would put the Y axis in the centre of the 
    /// screen. Unaffected by scaling.
    pan_x: Number,

    /// The Y offset of the viewport in pixels, where 0 would put the X axis in the centre of the 
    /// screen. Unaffected by scaling.
    pan_y: Number,

    /// The X axis scaling as a multiplier. A value of 1 would map each pixel along the width of the
    /// screen to ascending integer values of X in the plot equation. Values greater than 1 stretch
    /// the graph out, while values less than 1 squish it.
    scale_x: Number,

    /// The unstructured node tree which the user input to get the current value of `scale_x`.
    scale_x_tree: UnstructuredNodeRoot,

    /// The Y axis scaling as a multiplier. A value of 1 would map each pixel along the height of
    /// the screen to ascending integer values of Y. Values greater than 1 stretch the graph out,
    /// while values less than 1 squish it.
    scale_y: Number,

    /// The unstructured node tree which the user input to get the current value of `scale_y`.
    scale_y_tree: UnstructuredNodeRoot,
}

impl ViewWindow {
    /// Returns an initial view window, with no scaling or panning.
    fn new() -> ViewWindow {
        ViewWindow {
            pan_x: Number::zero(),
            pan_y: Number::zero(),
            scale_x: Number::one(),
            scale_x_tree: UnstructuredNodeRoot {
                root: UnstructuredNodeList {
                    items: vec![
                        UnstructuredNode::Token(Token::Digit(1)),
                    ]
                }
            },
            scale_y: Number::one(),
            scale_y_tree: UnstructuredNodeRoot {
                root: UnstructuredNodeList {
                    items: vec![
                        UnstructuredNode::Token(Token::Digit(1)),
                    ]
                }
            },
        }
    }

    /// Returns the screen position of the origin (0, 0) in the graph space.
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

/// A plot on the graph space, derived from an equation entered as an rbop node tree.
struct Plot {
    /// The unstructured node tree, as entered by the user to construct the graph.
    unstructured: UnstructuredNodeRoot,

    /// The structured node tree, as upgraded from the unstructured node tree. If the `unstructured`
    /// field is modified, this should be modified too to match.
    structured: StructuredNode,

    /// A calculated list of points on this graph. Each index is an X value on the *screen* (not the
    /// graph space), and the value is the corresponding Y value on the *graph space* (not the
    /// screen).
    y_values: Vec<Result<Number, MathsError>>,
}

impl Plot {
    /// Recalculates all of the `y_values` given a viewport and settings to evaluate with.
    fn recalculate_values(&mut self, view: &ViewWindow, settings: &EvaluationSettings) {
        self.y_values = view.x_coords_on_screen()
            .iter().map(|i| Self::calculate_one_value(*i, &self.structured, settings)).collect::<Vec<_>>();
    }

    /// Calculates one value for `y_values`, given an X value on the graph space, a node tree to
    /// evaluate, and settings to evaluate with.
    fn calculate_one_value(x: Number, node: &StructuredNode, settings: &EvaluationSettings) -> Result<Number, MathsError> {
        let sn_clone = node.substitute_variable(
            'x',
            &StructuredNode::Number(x)
        );
        sn_clone.evaluate(settings)
    }

    /// Recalculates a slice of `y_values` by "panning" the list of calculated values.
    /// 
    /// If the pan value is positive, this represents a pan of the viewport right. Calculated values
    /// are moved to lower indices of the list, and new values are calculated at the end to fill in
    /// the gap.
    /// 
    /// If the pan value is negative, this represents a pan of the viewport left. Calculated values
    /// are moved to higher indices of the list, and new values are calculated at the start to fill
    /// in the gap.
    /// 
    /// A zero value makes no change.
    fn recalculate_x_pan(&mut self, pan: isize, view: &ViewWindow, settings: &EvaluationSettings) {
        let x_values = view.x_coords_on_screen();

        if pan > 0 {
            // Moving right - copy values down
            let pan = pan as usize;
            for i in 0..(self.y_values.len() - pan) {
                self.y_values[i] = self.y_values[i + pan].clone();
            }

            // Insert new values
            for i in (self.y_values.len() - pan)..self.y_values.len() {
                self.y_values[i] = Self::calculate_one_value(x_values[i], &self.structured, settings);
            }
        } else if pan < 0 {
            // Moving left - copy values up
            let pan = pan.abs() as usize;
            for i in (0..(self.y_values.len() - pan)).rev() {
                self.y_values[i + pan] = self.y_values[i].clone();
            }
            
            // Insert new values
            for i in 0..pan {
                self.y_values[i] = Self::calculate_one_value(x_values[i], &self.structured, settings);
            }
        }
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
                        plot.recalculate_x_pan(-Self::PAN_AMOUNT, &self.view_window, &settings);
                    }
                },
                OSInput::Button(ButtonInput::MoveRight) => {
                    self.view_window.pan_x -= pan_amount;
                    let settings = self.settings();
                    for plot in &mut self.plots {
                        plot.recalculate_x_pan(Self::PAN_AMOUNT, &self.view_window, &settings);
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

        // Draw each graph from computed points
        for plot in &self.plots {
            let mut next_y_screen = self.view_window.y_to_screen(plot.y_values[0].clone().unwrap_or(Number::zero()));
            for this_x in 0..(plot.y_values.len() - 1) {
                let next_x = this_x + 1;

                plot.y_values[this_x].as_ref().unwrap();
                if let Ok(this_y) = plot.y_values[this_x] {
                    let next_y = plot.y_values[next_x].as_ref().unwrap_or(&this_y);
                    let this_y_screen = next_y_screen;
                    next_y_screen = self.view_window.y_to_screen(*next_y);
        
                    if this_y_screen == next_y_screen {
                        self.os_mut().display_sprite.draw_pixel(this_x as i16, this_y_screen, Colour::WHITE);
                    } else {
                        self.os_mut().display_sprite.draw_line(
                            this_x as i16, this_y_screen,
                            this_x as i16, next_y_screen,
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
        ContextMenu::new(
            self.os,
            vec![
                ContextMenuItem::new_common("Plots...", |this: &mut Self| {
                    this.draw();
                    this.plot_menu();
                }),
                ContextMenuItem::new_common("View window...", |this: &mut Self| {
                    this.draw();
                    this.view_window_menu();
                }),
            ],
            true,
        ).tick_until_call(self);

        self.draw();
    }

    fn plot_menu(&mut self) {
        self.draw();

        // Start with the menu item to add a new plot, then a divider
        let mut menu_items = vec![
            ContextMenuItem::new_common("Add plot", |this: &mut Self| {
                let (structured, unstructured) = this.input_expression_until_upgrade(None);
                let mut plot = Plot {
                    unstructured,
                    structured,
                    y_values: Vec::new()
                };
                plot.recalculate_values(&this.view_window, &this.settings());

                // Create and push plot
                this.plots.push(plot);
            }),
            ContextMenuItem::Divider,
        ];

        // Add an item to edit each existing plot
        for (i, _plot) in self.plots.iter().enumerate() {
            menu_items.push(
                ContextMenuItem::new_common(format!("Plot {}...", i), move |this: &mut Self| {
                    this.plot_edit_menu(i);
                }),
            )
        }

        ContextMenu::new(
            self.os,
            menu_items,
            true,
        ).tick_until_call(self);
    }

    fn plot_edit_menu(&mut self, plot_index: usize) {
        self.draw();

        ContextMenu::new(
            self.os,
            vec![
                ContextMenuItem::new_common("Edit", move |this: &mut Self| {
                    let (structured, unstructured) = this.input_expression_until_upgrade(
                        Some(this.plots[plot_index].unstructured.clone())
                    );
                    let settings = this.settings();
                    let plot = &mut this.plots[plot_index];
                    plot.unstructured = unstructured;
                    plot.structured = structured;
                    plot.recalculate_values(&this.view_window, &settings);    
                }),
                ContextMenuItem::new_common("Delete", move |this: &mut Self| {
                    this.plots.remove(plot_index);
                }),
            ],
            true,
        ).tick_until_call(self);
    }
    
    fn view_window_menu(&mut self) {
        self.draw();

        ContextMenu::new(
            self.os,
            vec![
                ContextMenuItem::new_common("X scale", |this: &mut Self| {
                    (this.view_window.scale_x, this.view_window.scale_x_tree) =
                        this.os_mut().ui_input_expression_and_evaluate(
                            "X scale:",
                            Some(this.view_window.scale_x_tree.clone()),
                            || (),
                        );

                    let settings = this.settings();
                    for plot in &mut this.plots {
                        plot.recalculate_values(&this.view_window, &settings);
                    }
                }),
                ContextMenuItem::new_common("Y scale", |this: &mut Self| {
                    (this.view_window.scale_y, this.view_window.scale_y_tree) =
                        this.os_mut().ui_input_expression_and_evaluate(
                            "Y scale:",
                            Some(this.view_window.scale_y_tree.clone()),
                            || (),
                        );

                    let settings = this.settings();
                    for plot in &mut this.plots {
                        plot.recalculate_values(&this.view_window, &settings);
                    }
                }),
            ],
            true,
        ).tick_until_call(self);
    }

    /// Returns the settings used for evaluating values for this graph. Notably, this sets the 
    /// `use_floats` flag, which makes evaluation use faster albeit less accurate computations.
    fn settings(&self) -> EvaluationSettings {
        EvaluationSettings {
            use_floats: true,
            ..self.os().filesystem.settings.evaluation_settings()
        }
    }

    fn input_expression_until_upgrade(&mut self, start: Option<UnstructuredNodeRoot>) -> (StructuredNode, UnstructuredNodeRoot) {
        loop {
            let unstructured = self.os_mut().ui_input_expression("y =", start.clone());
            match unstructured.upgrade() {
                Ok(s) => return (s, unstructured),
                Err(e) => {
                    self.os_mut().ui_text_dialog(&e.to_string());
                    self.draw();
                }
            }
        }
    }
}
