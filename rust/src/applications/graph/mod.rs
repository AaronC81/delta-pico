use alloc::{format, vec, vec::Vec, string::{ToString, String}, boxed::Box};
use num_traits::FromPrimitive;
use rbop::{Number, StructuredNode, node::{unstructured::{Upgradable, UnstructuredNodeRoot}, structured::EvaluationSettings, compiled::CompiledNode}, error::MathsError, render::{Viewport, Area}};
use rust_decimal::{prelude::{One, ToPrimitive, Zero}, Decimal};

use crate::{interface::{Colour, ApplicationFramework, ButtonInput, DISPLAY_WIDTH, DISPLAY_HEIGHT}, operating_system::{OSInput, OperatingSystem, os_accessor, OperatingSystemPointer, ContextMenu, ContextMenuItem, SelectorMenuCallable}, rbop_impl::RbopSpriteRenderer};
use super::{Application, ApplicationInfo};

mod test;

/// Represents the current viewport position and scale.
pub struct CalculatedViewWindow {
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

    /// The Y axis scaling as a multiplier. A value of 1 would map each pixel along the height of
    /// the screen to ascending integer values of Y. Values greater than 1 stretch the graph out,
    /// while values less than 1 squish it.
    scale_y: Number,
}

impl CalculatedViewWindow {
    /// Returns the screen position of the origin (0, 0) in the graph space.
    fn axis_screen_coords(&self) -> (i16, i16) {
        (
            self.x_to_screen(Number::zero()).unwrap(),
            self.y_to_screen(Number::zero()).unwrap(),
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
    /// 
    /// If the X value is out of the range of an i16, returns None.
    fn x_to_screen(&self, mut x: Number) -> Option<i16> {
        // Apply scale
        x *= self.scale_x;

        // Apply user-specified pan
        x += self.pan_x;

        // Squash into an integer, and pan so that (0, 0) is in the middle of
        // the screen
        x.to_decimal().to_i16().map(|x| x.saturating_add(DISPLAY_WIDTH as i16 / 2))
    }

    /// Given a Y value in the graph space, returns a Y value on the screen.
    /// 
    /// If the Y value is out of the range of an i16, returns None.
    fn y_to_screen(&self, mut y: Number) -> Option<i16> {
        // Apply scale
        y *= self.scale_y;

        // Apply user-specified pan
        y += self.pan_y;

        // Squash into an integer, flip around the bottom of the screen, and
        // pan so that (0, 0) is in the middle of the screen
        y.to_decimal().to_i16().map(|y| (DISPLAY_HEIGHT as i16 / 2).saturating_sub(y))
    }
}

/// Represents the user input which was used to calculate the view window, with minimum and maximum
/// values for each axis of the graph space.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct UserViewWindow {
    x_min: Number,
    x_max: Number,
    y_min: Number,
    y_max: Number,
}

impl UserViewWindow {
    fn new() -> Self {
        Self {
            x_min: Number::from(-10),
            x_max: Number::from(10),
            y_min: Number::from(-10),
            y_max: Number::from(10),
        }
    }

    fn to_calculated(&self) -> CalculatedViewWindow {
        let scale_x = Number::from(DISPLAY_WIDTH as i64) / (self.x_max - self.x_min);
        let pan_x = -((self.x_max + self.x_min) / 2.into()) * scale_x;
        
        let scale_y = Number::from(DISPLAY_HEIGHT as i64) / (self.y_max - self.y_min);
        let pan_y = -((self.y_max + self.y_min) / 2.into()) * scale_y;

        CalculatedViewWindow { pan_x, pan_y, scale_x, scale_y }
    }
}

/// A plot on the graph space, derived from an equation entered as an rbop node tree.
struct Plot {
    /// The unstructured node tree, as entered by the user to construct the graph.
    unstructured: UnstructuredNodeRoot,

    /// The compiled node tree, as upgraded and compiled from the unstructured node tree. If the
    /// `unstructured` field is modified, this should be modified too to match.
    compiled: CompiledNode,

    /// A calculated list of points on this graph. Each index is an X value on the *screen* (not the
    /// graph space), and the value is the corresponding Y value on both the graph space and the 
    /// screen (in that order).
    y_values: Vec<Result<(Number, i16), MathsError>>,
}

impl Plot {
    /// Recalculates all of the `y_values` given a viewport and settings to evaluate with.
    fn recalculate_values(&mut self, view: &CalculatedViewWindow) {
        self.y_values = view.x_coords_on_screen()
            .iter().map(|i| Self::calculate_one_value(*i, &self.compiled, view)).collect::<Vec<_>>();
    }

    /// Calculates one value for `y_values`, given an X value on the graph space, a node tree to
    /// evaluate, and settings to evaluate with.
    fn calculate_one_value(x: Number, node: &CompiledNode, view: &CalculatedViewWindow) -> Result<(Number, i16), MathsError> {
        let real_value = node.evaluate_raw(x)?;
        let screen_value = view.y_to_screen(real_value).ok_or(MathsError::Overflow)?;

        Ok((real_value, screen_value))
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
    fn recalculate_x_pan(&mut self, pan: isize, view: &CalculatedViewWindow) {
        let x_values = view.x_coords_on_screen();

        if pan > 0 {
            // Moving right - copy values down
            let pan = pan as usize;
            for i in 0..(self.y_values.len() - pan) {
                self.y_values[i] = self.y_values[i + pan].clone();
            }

            // Insert new values
            for i in (self.y_values.len() - pan)..self.y_values.len() {
                self.y_values[i] = Self::calculate_one_value(x_values[i], &self.compiled, view);
            }
        } else if pan < 0 {
            // Moving left - copy values up
            let pan = pan.abs() as usize;
            for i in (0..(self.y_values.len() - pan)).rev() {
                self.y_values[i + pan] = self.y_values[i].clone();
            }
            
            // Insert new values
            for i in 0..pan {
                self.y_values[i] = Self::calculate_one_value(x_values[i], &self.compiled, view);
            }
        }
    }

    /// Recalculates the screen positions of `y_values` by "panning" the list of calculated values.
    /// Each calculated screen position has the pan amount added to it.
    fn recalculate_y_pan(&mut self, pan: i16, _view: &CalculatedViewWindow) {
        for item in &mut self.y_values {
            if let Ok((_, screen)) = item {
                *screen += pan;
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MovementMode {
    Freeform,
    Trace(TraceState),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TraceState {
    plot_index: usize,
    current_x: Number,
}

impl TraceState {
    fn new(view: &UserViewWindow, plot_index: usize) -> Self {
        Self {
            plot_index,
            current_x: (view.x_max + view.x_min).to_decimal_number() / Decimal::TWO.into(),
        }
    }

    const TRACE_INCREMENTS_PER_SCREEN: usize = 20;

    fn x_increment(view: &UserViewWindow) -> Number {
        (view.x_max - view.x_min).to_decimal_number() / Number::from(Self::TRACE_INCREMENTS_PER_SCREEN as i64)
    }

    fn pan_for_current_x(&self, user_view: &mut UserViewWindow, calc_view: &mut CalculatedViewWindow, plots: &mut [Plot]) {
        let inc = Self::x_increment(user_view);
        let mut need_recalc = false;

        if (user_view.x_min - self.current_x).abs() < inc * Decimal::from_f32(1.5).unwrap().into() {
            user_view.x_min -= inc;
            user_view.x_max -= inc;
            need_recalc = true;
        }

        if (user_view.x_max - self.current_x).abs() < inc * Decimal::from_f32(1.5).unwrap().into() {
            user_view.x_min += inc;
            user_view.x_max += inc;
            need_recalc = true;
        }

        if need_recalc {
            *calc_view = user_view.to_calculated();
            for plot in plots {
                // TODO: can we partially recalculate like with freeform pans?
                plot.recalculate_values(calc_view);
            }
        }
    }
}

pub struct GraphApplication<F: ApplicationFramework + 'static> {
    os: OperatingSystemPointer<F>,
    plots: Vec<Plot>,
    user_view_window: UserViewWindow,
    calculated_view_window: CalculatedViewWindow,
    movement_mode: MovementMode,
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
        let user_view_window = UserViewWindow::new();

        Self {
            os,
            plots: Vec::new(),
            user_view_window,
            calculated_view_window: user_view_window.to_calculated(),
            movement_mode: MovementMode::Freeform,
        }
    }

    fn tick(&mut self) {
        self.draw();

        // Poll for input
        if let Some(input) = self.os_mut().input() {
            let pan_amount = Number::from(Self::PAN_AMOUNT as i64);
            let is_freeform = self.movement_mode == MovementMode::Freeform;
            let is_trace = matches!(self.movement_mode, MovementMode::Trace(_));
            match input {
                // Freeform movement
                OSInput::Button(ButtonInput::MoveLeft) if is_freeform => {
                    self.user_view_window.x_min -= pan_amount / self.calculated_view_window.scale_x;
                    self.user_view_window.x_max -= pan_amount / self.calculated_view_window.scale_x;
                    self.calculated_view_window = self.user_view_window.to_calculated();

                    for plot in &mut self.plots {
                        plot.recalculate_x_pan(-Self::PAN_AMOUNT, &self.calculated_view_window);
                    }
                },
                OSInput::Button(ButtonInput::MoveRight) if is_freeform => {
                    self.user_view_window.x_min += pan_amount / self.calculated_view_window.scale_x;
                    self.user_view_window.x_max += pan_amount / self.calculated_view_window.scale_x;
                    self.calculated_view_window = self.user_view_window.to_calculated();

                    for plot in &mut self.plots {
                        plot.recalculate_x_pan(Self::PAN_AMOUNT, &self.calculated_view_window);
                    }
                }
                OSInput::Button(ButtonInput::MoveUp) if is_freeform => {
                    self.user_view_window.y_min += pan_amount / self.calculated_view_window.scale_y;
                    self.user_view_window.y_max += pan_amount / self.calculated_view_window.scale_y;
                    self.calculated_view_window = self.user_view_window.to_calculated();

                    for plot in &mut self.plots {
                        plot.recalculate_y_pan(Self::PAN_AMOUNT as i16, &self.calculated_view_window);
                    }
                }
                OSInput::Button(ButtonInput::MoveDown) if is_freeform => {
                    self.user_view_window.y_min -= pan_amount / self.calculated_view_window.scale_y;
                    self.user_view_window.y_max -= pan_amount / self.calculated_view_window.scale_y;
                    self.calculated_view_window = self.user_view_window.to_calculated();

                    for plot in &mut self.plots {
                        plot.recalculate_y_pan(-Self::PAN_AMOUNT as i16, &self.calculated_view_window);
                    }
                }

                // Trace movement
                OSInput::Button(ButtonInput::MoveLeft) if is_trace => {
                    if let MovementMode::Trace(ref mut state) = self.movement_mode {
                        state.current_x -= TraceState::x_increment(&self.user_view_window);
                        state.pan_for_current_x(
                            &mut self.user_view_window,
                            &mut self.calculated_view_window,
                            &mut self.plots[..],
                        );
                    } else {
                        unreachable!()
                    }
                },
                OSInput::Button(ButtonInput::MoveRight) if is_trace => {
                    if let MovementMode::Trace(ref mut state) = self.movement_mode {
                        state.current_x += TraceState::x_increment(&self.user_view_window);
                        state.pan_for_current_x(
                            &mut self.user_view_window,
                            &mut self.calculated_view_window,
                            &mut self.plots[..],
                        );
                    } else {
                        unreachable!()
                    }
                }
                OSInput::Button(ButtonInput::MoveUp) if is_trace => {
                    if let MovementMode::Trace(ref mut state) = self.movement_mode {
                        if state.plot_index == 0 {
                            state.plot_index = self.plots.len() - 1;
                        } else {
                            state.plot_index -= 1;
                        }
                    } else {
                        unreachable!()
                    }
                }
                OSInput::Button(ButtonInput::MoveDown) if is_trace => {
                    if let MovementMode::Trace(ref mut state) = self.movement_mode {
                        state.plot_index += 1;
                        state.plot_index %= self.plots.len();
                    } else {
                        unreachable!()
                    }
                }

                // Available everywhere
                OSInput::Button(ButtonInput::Exe) => {
                    match self.movement_mode {
                        // Don't transition to trace mode if there are no plots
                        MovementMode::Freeform if self.plots.is_empty() => (),

                        MovementMode::Freeform => self.movement_mode = MovementMode::Trace(TraceState::new(&self.user_view_window, 0)),
                        MovementMode::Trace(_) => self.movement_mode = MovementMode::Freeform,
                    }
                }
                OSInput::Button(ButtonInput::List) => self.open_menu(),

                _ => (),
            }
        }
    }

    fn test(&mut self) {
        test::test(self);
    }
}

impl<F: ApplicationFramework> GraphApplication<F> {
    const PAN_AMOUNT: isize = 10;

    fn draw(&mut self) {
        self.os_mut().display_sprite.fill(Colour::BLACK);

        // Draw axes
        let (x_axis, y_axis) = self.calculated_view_window.axis_screen_coords();
        self.os_mut().display_sprite.draw_line(x_axis, 0, x_axis, DISPLAY_HEIGHT as i16, Colour::BLUE);
        self.os_mut().display_sprite.draw_line(0, y_axis, DISPLAY_WIDTH as i16, y_axis, Colour::BLUE);

        // Draw each graph from computed points
        for plot in &self.plots {
            for this_x in 0..(plot.y_values.len() - 1) {
                if let Ok(this_y) = plot.y_values[this_x] && let Ok(next_y) = plot.y_values[this_x + 1] {
                    let this_y_screen = this_y.1;
                    let next_y_screen = next_y.1;
        
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

        // If tracing...
        if let MovementMode::Trace(state) = self.movement_mode {
            // Work out current Y
            // TODO: don't hardcode to first plot
            let current_y = self.plots[state.plot_index].compiled.evaluate_raw(state.current_x);

            // Print current coordinates
            self.os_mut().display_sprite.print_at(
                0, 0,
                &format!("X: {}\nY: {}",
                    state.current_x.to_decimal_number().simplify().to_decimal(),
                    match current_y {
                        Ok(num) => num.to_decimal_number().simplify().to_decimal().to_string(),
                        Err(ref e) => e.to_string(),
                    },
                )
            );

            // Draw a marker where we are right now
            let screen_x = self.calculated_view_window.x_to_screen(state.current_x);
            let screen_y = current_y.map(|y| self.calculated_view_window.y_to_screen(y));
            if let Some(screen_x) = screen_x && let Ok(Some(screen_y)) = screen_y {
                for i in -4..=4 {
                    self.os_mut().display_sprite.draw_pixel(screen_x + i, screen_y + i, Colour::ORANGE);
                    self.os_mut().display_sprite.draw_pixel(screen_x - i, screen_y + i, Colour::ORANGE);
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
                if let Some((structured, unstructured)) = this.input_expression_until_upgrade(None) {
                    let compiled = CompiledNode::from_structured(structured, Some('x'), &this.settings());
                    let mut plot = Plot {
                        unstructured,
                        compiled,
                        y_values: Vec::new()
                    };
                    plot.recalculate_values(&this.calculated_view_window);

                    // Create and push plot
                    this.plots.push(plot);
                }
            }),
            ContextMenuItem::Divider,
        ];

        // Add an item to edit each existing plot
        for (i, plot) in self.plots.iter_mut().enumerate() {
            let viewport = Viewport::new(Area::new(DISPLAY_WIDTH as u64 - 10, 100));
            let sprite = RbopSpriteRenderer::draw_to_sprite(
                &mut plot.unstructured,
                None,
                Some(&viewport),
                Colour::GREY,
            );
            let selected_sprite = RbopSpriteRenderer::draw_to_sprite(
                &mut plot.unstructured,
                None,
                Some(&viewport),
                Colour::BLUE,
            );
            menu_items.push(
                ContextMenuItem::Sprite {
                    sprite,
                    selected_sprite: Some(selected_sprite),
                    metadata: Box::new(move |this: &mut Self| {
                        this.plot_edit_menu(i);
                    })
                }
            );
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
                    if let Some((structured, unstructured)) = this.input_expression_until_upgrade(
                        Some(this.plots[plot_index].unstructured.clone())
                    )
                    {
                        let settings = this.settings();
                        let plot = &mut this.plots[plot_index];
                        let compiled = CompiledNode::from_structured(structured, Some('x'), &settings);
                        plot.unstructured = unstructured;
                        plot.compiled = compiled;
                        plot.recalculate_values(&this.calculated_view_window);    
                    }
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

        fn truncate_string(str: String, limit: usize) -> String {
            if str.len() > limit {
                str.chars().take(limit).chain("...".chars()).collect()
            } else {
                str
            }
        }

        macro_rules! vw_edit {
            ($label: expr, $this: ident, $accessor: expr, $outer_accessor: expr) => {
                ContextMenuItem::new_common(
                    format!("{} = {}",
                        $label,
                        truncate_string($outer_accessor.to_decimal().to_string(), 10),
                    ),
                    |$this: &mut Self| {
                        if let Some((x, _)) =
                                $this.os_mut().ui_input_expression_and_evaluate(
                                    $label,
                                    Some(UnstructuredNodeRoot::from_number($accessor)),
                                    || (),
                                )
                        {
                            $accessor = x;
                            $this.recalculate_all();
                        }
                    },
                )
            };
        }

        ContextMenu::new(
            self.os,
            vec![
                ContextMenuItem::new_common("Auto view", |this: &mut Self| {
                    this.auto_view();
                }),
                vw_edit!("X min.", this, this.user_view_window.x_min, self.user_view_window.x_min),
                vw_edit!("X max.", this, this.user_view_window.x_max, self.user_view_window.x_max),
                vw_edit!("Y min.", this, this.user_view_window.y_min, self.user_view_window.y_min),
                vw_edit!("Y max.", this, this.user_view_window.y_max, self.user_view_window.y_max),
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

    /// Repeatedly prompts the user to input an expression until it upgrades successfully, then
    /// returns it.
    /// 
    /// If the user opens the menu, returns `None`.
    fn input_expression_until_upgrade(&mut self, start: Option<UnstructuredNodeRoot>) -> Option<(StructuredNode, UnstructuredNodeRoot)> {
        loop {
            if let Some(unstructured) = self.os_mut().ui_input_expression("y =", start.clone()) {
                match unstructured.upgrade() {
                    Ok(s) => return Some((s, unstructured)),
                    Err(e) => {
                        self.os_mut().ui_text_dialog(&e.to_string());
                        self.draw();
                    }
                }
            } else {
                return None;
            }
        }
    }

    /// Adjusts the view window to attempt to best display the plots on the screen.
    fn auto_view(&mut self) {
        // Find the min and max Y values within the X boundaries of the screen
        let mut sorted_y_values = self.plots.iter()
            .flat_map(|p| p.y_values.iter().filter_map(|x| x.as_ref().ok()))
            .collect::<Vec<_>>();
        sorted_y_values.sort_unstable();
        let min_y_graph_value = sorted_y_values[0].0;
        let max_y_graph_value = sorted_y_values.last().unwrap().0;
        
        self.user_view_window.y_min = min_y_graph_value;
        self.user_view_window.y_max = max_y_graph_value;
        self.recalculate_all();
    }

    fn recalculate_all(&mut self) {
        // Recalculate view window
        self.calculated_view_window = self.user_view_window.to_calculated();

        // Adjust plots
        for plot in &mut self.plots {
            plot.recalculate_values(&self.calculated_view_window);
        }
    }
}
