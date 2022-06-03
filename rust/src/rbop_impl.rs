use alloc::{string::ToString, vec};
use az::SaturatingAs;
use rbop::{Token, UnstructuredNode, UnstructuredNodeList, nav::{MoveVerticalDirection, NavPath}, node::{unstructured::{UnstructuredNodeRoot, MoveResult}, function::Function}, render::{Area, Glyph, Renderer, Viewport, ViewportGlyph, ViewportVisibility, LayoutComputationProperties, Layoutable}};
use crate::{interface::{Colour, ShapeFill, ButtonInput, ApplicationFramework}, operating_system::{OSInput, OperatingSystem, os_accessor}, graphics::Sprite};

use core::cmp::max;

pub struct RbopContext<F: ApplicationFramework + 'static> {
    pub os: *mut OperatingSystem<F>,

    pub root: UnstructuredNodeRoot,
    pub nav_path: NavPath,
    pub viewport: Option<Viewport>,

    pub input_shift: bool,
}

os_accessor!(RbopContext<F>);

impl<F: ApplicationFramework> RbopContext<F> {
    pub fn new(os: *mut OperatingSystem<F>) -> RbopContext<F> {
        RbopContext {
            os,
            root: UnstructuredNodeRoot { root: UnstructuredNodeList { items: vec![] } },
            nav_path: NavPath::new(vec![0]),
            viewport: None,
            input_shift: false,
        }
    }

    pub fn input(&mut self, input: OSInput) -> Option<(MoveVerticalDirection, MoveResult)> {
        let mut renderer = RbopSpriteRenderer::new();

        let node_to_insert = if !self.input_shift {
            match input {        
                OSInput::Button(ButtonInput::MoveLeft) => {
                    self.root.move_left(&mut self.nav_path, &mut renderer, self.viewport.as_mut());
                    None
                }
                OSInput::Button(ButtonInput::MoveRight) => {
                    self.root.move_right(&mut self.nav_path, &mut renderer, self.viewport.as_mut());
                    None
                }
                OSInput::Button(ButtonInput::MoveUp) => {
                    return Some((
                        MoveVerticalDirection::Up,
                        self.root.move_up(&mut self.nav_path, &mut renderer, self.viewport.as_mut())
                    ));
                }
                OSInput::Button(ButtonInput::MoveDown) => {
                    return Some((
                        MoveVerticalDirection::Down,
                        self.root.move_down(&mut self.nav_path, &mut renderer, self.viewport.as_mut())
                    ));
                }
                OSInput::Button(ButtonInput::Delete) => {
                    self.root.delete(&mut self.nav_path, &mut renderer, self.viewport.as_mut());
                    None
                }
                OSInput::Button(ButtonInput::Clear) => {
                    self.root.clear(&mut self.nav_path, &mut renderer, self.viewport.as_mut());
                    None
                }
        
                OSInput::Button(ButtonInput::Digit(d)) => Some(UnstructuredNode::Token(Token::Digit(d))),
        
                OSInput::Button(ButtonInput::Point) => Some(UnstructuredNode::Token(Token::Point)),
                OSInput::Button(ButtonInput::Parentheses) => Some(UnstructuredNode::Parentheses(
                    UnstructuredNodeList { items: vec![] },
                )),
        
                OSInput::Button(ButtonInput::Add) => Some(UnstructuredNode::Token(Token::Add)),
                OSInput::Button(ButtonInput::Subtract) => Some(UnstructuredNode::Token(Token::Subtract)),
                OSInput::Button(ButtonInput::Multiply) => Some(UnstructuredNode::Token(Token::Multiply)),
                OSInput::Button(ButtonInput::Fraction) => Some(UnstructuredNode::Fraction(
                    UnstructuredNodeList { items: vec![] },
                    UnstructuredNodeList { items: vec![] },
                )),
                OSInput::Button(ButtonInput::Power) => Some(UnstructuredNode::Power(
                    UnstructuredNodeList { items: vec![] },
                )),
                OSInput::Button(ButtonInput::Sqrt) => Some(UnstructuredNode::Sqrt(
                    UnstructuredNodeList { items: vec![] },
                )),

                OSInput::TextMultiTapNew(c) => Some(UnstructuredNode::Token(Token::Variable(c))),
                OSInput::TextMultiTapCycle(c) => {
                    self.root.delete(&mut self.nav_path, &mut renderer, self.viewport.as_mut());
                    Some(UnstructuredNode::Token(Token::Variable(c)))
                }

                OSInput::Button(ButtonInput::Exe) => return None,
                OSInput::Button(ButtonInput::List) => return None,
                OSInput::Button(ButtonInput::Shift) => {
                    // TODO: should there just be one shift?
                    if self.os().text_mode {
                        self.os_mut().multi_tap.shift = true;
                    } else {
                        self.input_shift = true;
                    }
                    None
                }

                _ => todo!(),
            }
        } else {
            let mut input_pressed = true;
            let node = match input {
                OSInput::Button(ButtonInput::Shift) => {
                    self.input_shift = false;
                    None
                }
                OSInput::Button(ButtonInput::Digit(0)) => Some(UnstructuredNode::Token(Token::Variable('x'))),
                OSInput::Button(ButtonInput::Digit(1)) => Some(UnstructuredNode::FunctionCall(Function::Sine, vec![UnstructuredNodeList::new()])),
                OSInput::Button(ButtonInput::Digit(2)) => Some(UnstructuredNode::FunctionCall(Function::Cosine, vec![UnstructuredNodeList::new()])),

                _ => {
                    input_pressed = false;
                    None
                },
            };

            if input_pressed {
                self.input_shift = false;
            }

            node
        };
    
        if let Some(node) = node_to_insert {
            self.root.insert(&mut self.nav_path, &mut renderer, self.viewport.as_mut(), node);
        }

        None
    }
}

pub struct RbopSpriteRenderer {
    sprite: Option<Sprite>,
}

impl RbopSpriteRenderer {
    pub fn new() -> Self {
        RbopSpriteRenderer { sprite: None }
    }

    pub fn draw_context_to_sprite<F: ApplicationFramework>(rbop_ctx: &mut RbopContext<F>, background_colour: Colour) -> Sprite {
        Self::draw_to_sprite::<F, UnstructuredNodeRoot>(
            &mut rbop_ctx.root,
            Some(&mut rbop_ctx.nav_path),
            rbop_ctx.viewport.as_ref(),
            background_colour
        )
    }

    pub fn draw_to_sprite<F: ApplicationFramework, N: Layoutable>(
        node: &mut N,
        mut nav_path: Option<&mut NavPath>,
        viewport: Option<&Viewport>,
        background_colour: Colour
    ) -> Sprite {
        let mut renderer = Self::new();

        // Calculate layout in advance so we know size
        let mut navigator = nav_path.as_mut().map(|nav_path| nav_path.to_navigator());
        let layout = renderer.layout(
            node,
            navigator.as_mut(),
            LayoutComputationProperties::default(),
        );
        let height: u16 = layout.area.height.saturating_as::<u16>();
        let width = layout.area.width.saturating_as::<u16>();

        // Create sprite now that we know the size, and draw its background
        // 3 larger to account for the possibility that the cursor is at the end - we told rbop
        // that the cursor has a width of 0, so it won't account for it in the layout size, and
        // various other lies told by nodes
        let mut sprite = Sprite::new(width + 3, height + 3);
        sprite.fill(background_colour);
        renderer.sprite = Some(sprite);
        
        // Draw background and expression to sprite
        let mut navigator = nav_path.as_mut().map(|nav_path| nav_path.to_navigator());
        renderer.draw_all(
            node, 
            navigator.as_mut(),
            viewport,
        );

        renderer.sprite.unwrap()
    }
}

impl Default for RbopSpriteRenderer {
    fn default() -> Self {
        Self::new()
    }
}

const MINIMUM_PAREN_HEIGHT: u64 = 16;

impl Renderer for RbopSpriteRenderer {
    fn size(&mut self, glyph: Glyph, size_reduction_level: u32) -> Area {        
        // Size calculation doesn't need an actual sprite to draw on, so if one hasn't been supplied
        // yet, just use a blank one
        let mut blank_sprite = Sprite::new(0, 0);
        let mut sprite = self.sprite.as_mut().unwrap_or(&mut blank_sprite);

        // If there is any size reduction level, set a smaller font
        let mut restore_font = None;
        if size_reduction_level > 0 {
            restore_font = Some(sprite.font);
            sprite.font = &crate::font_data::DroidSans14;
        }

        // Calculate an average character size
        let (text_character_width, text_character_height) = sprite.font.string_size("0");
        let text_character_size = Area {
            width: text_character_width as u64,
            height: text_character_height as u64,
        };

        let result = match glyph {
            Glyph::Cursor { height } => Area { height, width: 0 },
            Glyph::Placeholder => text_character_size,

            Glyph::Digit { .. } => text_character_size,
            Glyph::Variable { name } => {
                let (width, height) = sprite.font.string_size(&name.to_string());
                Area {
                    width: width as u64,
                    height: height as u64,
                }
            },

            Glyph::Point => Area { width: text_character_size.width / 2, ..text_character_size },
            Glyph::Comma => text_character_size,
            Glyph::Add => text_character_size,
            Glyph::Subtract => text_character_size,
            Glyph::Multiply => text_character_size,
            Glyph::Divide => text_character_size,
            Glyph::Fraction { inner_width } => Area { height: 1, width: inner_width },

            Glyph::LeftParenthesis { inner_height } => Area { width: 5, height: max(inner_height, MINIMUM_PAREN_HEIGHT) },
            Glyph::RightParenthesis { inner_height } => Area { width: 5, height: max(inner_height, MINIMUM_PAREN_HEIGHT) },

            Glyph::Sqrt { inner_area } => Area { width: inner_area.width + 14, height: inner_area.height + 5 },

            Glyph::FunctionName { function } => {
                let (w, h) = sprite.font.string_size(function.render_name());
                Area::new(w as u64, h as u64)
            }
        };

        if let Some(restore_font) = restore_font {
            sprite.font = restore_font;
        }

        result
    }

    fn square_root_padding(&self) -> u64 { 5 }

    fn init(&mut self, _size: Area) {}

    fn draw(&mut self, mut glyph: ViewportGlyph) {        
        match glyph.visibility {
            ViewportVisibility::Clipped { invisible, .. } if invisible => return,
            ViewportVisibility::Clipped { left_clip, right_clip, .. } => {
                // Re-align and shorten a left-clipped fraction line
                if let Glyph::Fraction { inner_width } = glyph.glyph.glyph {
                    if left_clip > 0 {
                        glyph.glyph = Glyph::Fraction {
                            inner_width: inner_width - left_clip
                        }.to_sized(self, glyph.glyph.size_reduction_level);
                        glyph.point.x = 0;
                    }
                }

                // Shorten a right-clipped fraction line
                // (The if-let binding is repeated to get a possibly updated inner_width)
                if let Glyph::Fraction { inner_width } = glyph.glyph.glyph {
                    if right_clip > 0 {
                        glyph.glyph = Glyph::Fraction {
                            inner_width: inner_width - right_clip
                        }.to_sized(self, glyph.glyph.size_reduction_level);
                    }
                }                
            }
            _ => (),
        }

        let point = glyph.point;
        let (x, y) = (point.x.saturating_as::<i16>(), point.y.saturating_as::<i16>());
        
        // Drawing does need a sprite, so panic if we don't have one
        let sprite = self.sprite.as_mut().expect("RbopSpriteRenderer missing a sprite");

        // If there is any size reduction level, set a smaller font
        let mut restore_font = None;
        if glyph.glyph.size_reduction_level > 0 {
            restore_font = Some(sprite.font);
            sprite.font = &crate::font_data::DroidSans14;
        }

        match glyph.glyph.glyph {
            Glyph::Digit { number } => sprite.draw_char_at(x, y, (number + b'0') as char),
            Glyph::Point => sprite.draw_char_at(x, y, '.'),
            Glyph::Comma => sprite.draw_char_at(x, y, ','),
            Glyph::Variable { name } => sprite.draw_char_at(x, y, name),
            Glyph::Add => sprite.draw_char_at(x, y, '+'),
            Glyph::Subtract => sprite.draw_char_at(x, y, '-'),
            Glyph::Multiply => sprite.draw_char_at(x, y, '*'),
            Glyph::Divide => sprite.draw_char_at(x, y, '/'),
            
            Glyph::Fraction { inner_width } =>
                sprite.draw_line(x, y, x + inner_width.saturating_as::<i16>(), y, Colour::WHITE),
            
            Glyph::Cursor { height } =>
                sprite.draw_line(x, y, x, y + height.saturating_as::<i16>(), Colour::WHITE),

            Glyph::Placeholder => {
                let digit_glyph = sprite.font.char_data(b'0').unwrap();
                sprite.draw_rect(
                    x + 4, y + 4, (digit_glyph.width - 8).into(), (digit_glyph.height - 8).into(),
                    Colour::GREY, ShapeFill::Filled, 0
                );
            }

            Glyph::LeftParenthesis { inner_height } => {
                let inner_height = max(MINIMUM_PAREN_HEIGHT, inner_height).saturating_as::<i16>();
                
                sprite.draw_line(x + 3, y, x + 3, y + 1, Colour::WHITE);
                sprite.draw_line(x + 2, y + 2, x + 2, y + 6, Colour::WHITE);

                sprite.draw_line(x + 1, y + 7, x + 1, y + inner_height - 8, Colour::WHITE);

                sprite.draw_line(x + 3, y + inner_height - 2, x + 3, y + inner_height - 1, Colour::WHITE);
                sprite.draw_line(x + 2, y + inner_height - 7, x + 2, y + inner_height - 3, Colour::WHITE);
            }
            Glyph::RightParenthesis { inner_height } => {
                let inner_height = max(MINIMUM_PAREN_HEIGHT, inner_height).saturating_as::<i16>();
                
                sprite.draw_line(x + 1, y, x + 1, y + 1, Colour::WHITE);
                sprite.draw_line(x + 2, y + 2, x + 2, y + 6, Colour::WHITE);

                sprite.draw_line(x + 3, y + 7, x + 3, y + inner_height - 8, Colour::WHITE);

                sprite.draw_line(x + 1, y + inner_height - 2, x + 1, y + inner_height - 1, Colour::WHITE);
                sprite.draw_line(x + 2, y + inner_height - 7, x + 2, y + inner_height - 3, Colour::WHITE);
            }

            Glyph::Sqrt { inner_area } => {
                // Little line at the beginning
                sprite.draw_line(x, y + inner_area.height as i16 - 3, x + 4, y + inner_area.height as i16 + 1, Colour::WHITE);

                // Left line from bottom to top
                sprite.draw_line(x + 4, y + inner_area.height as i16 + 1, x + 7, y, Colour::WHITE);

                // Line along top
                sprite.draw_line(x + 7, y, x + inner_area.width as i16 + 10, y, Colour::WHITE);

                // Little flick at the end
                sprite.draw_line(x + inner_area.width as i16 + 10, y, x + inner_area.width as i16 + 10, y + 4, Colour::WHITE);
            },

            Glyph::FunctionName { function } => sprite.print_at(x, y, function.render_name())
        }

        if let Some(restore_font) = restore_font {
            sprite.font = restore_font;
        }
    }
}
