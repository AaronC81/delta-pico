use alloc::{string::ToString, vec};
use rbop::{Token, UnstructuredNode, UnstructuredNodeList, nav::{MoveVerticalDirection, NavPath}, node::unstructured::{UnstructuredNodeRoot, MoveResult}, render::{Area, Glyph, Renderer, Viewport, ViewportGlyph, ViewportVisibility}};
use crate::{interface::{ApplicationFrameworkInterface, Colour, ShapeFill, framework}, operating_system::{OSInput, os}};

use core::cmp::max;

pub struct RbopContext {
    pub root: UnstructuredNodeRoot,
    pub nav_path: NavPath,
    pub viewport: Option<Viewport>,

    pub input_shift: bool,
}

impl RbopContext {
    pub fn new() -> RbopContext {
        RbopContext {
            root: UnstructuredNodeRoot { root: UnstructuredNodeList { items: vec![] } },
            nav_path: NavPath::new(vec![0]),
            viewport: None,
            input_shift: false,
        }
    }

    pub fn input(&mut self, input: OSInput) -> Option<(MoveVerticalDirection, MoveResult)> {
        let renderer = framework();

        let node_to_insert = if !self.input_shift {
            match input {        
                OSInput::MoveLeft => {
                    self.root.move_left(&mut self.nav_path, renderer, self.viewport.as_mut());
                    None
                }
                OSInput::MoveRight => {
                    self.root.move_right(&mut self.nav_path, renderer, self.viewport.as_mut());
                    None
                }
                OSInput::MoveUp => {
                    return Some((
                        MoveVerticalDirection::Up,
                        self.root.move_up(&mut self.nav_path, renderer, self.viewport.as_mut())
                    ));
                }
                OSInput::MoveDown => {
                    return Some((
                        MoveVerticalDirection::Down,
                        self.root.move_down(&mut self.nav_path, renderer, self.viewport.as_mut())
                    ));
                }
                OSInput::Delete => {
                    self.root.delete(&mut self.nav_path, renderer, self.viewport.as_mut());
                    None
                }
        
                OSInput::Digit(d) => Some(UnstructuredNode::Token(Token::Digit(d))),
        
                OSInput::Point => Some(UnstructuredNode::Token(Token::Point)),
                OSInput::Parentheses => Some(UnstructuredNode::Parentheses(
                    UnstructuredNodeList { items: vec![] },
                )),
        
                OSInput::Add => Some(UnstructuredNode::Token(Token::Add)),
                OSInput::Subtract => Some(UnstructuredNode::Token(Token::Subtract)),
                OSInput::Multiply => Some(UnstructuredNode::Token(Token::Multiply)),
                OSInput::Fraction => Some(UnstructuredNode::Fraction(
                    UnstructuredNodeList { items: vec![] },
                    UnstructuredNodeList { items: vec![] },
                )),
                OSInput::Power => Some(UnstructuredNode::Power(
                    UnstructuredNodeList { items: vec![] },
                )),

                OSInput::TextMultiTapNew(c) => Some(UnstructuredNode::Token(Token::Variable(c))),
                OSInput::TextMultiTapCycle(c) => {
                    self.root.delete(&mut self.nav_path, renderer, self.viewport.as_mut());
                    Some(UnstructuredNode::Token(Token::Variable(c)))
                }

                OSInput::Exe => return None,
                OSInput::List => return None,
                OSInput::Shift => {
                    // TODO: should there just be one shift?
                    if os().text_mode {
                        os().multi_tap.shift = true;
                    } else {
                        self.input_shift = true;
                    }
                    None
                }
            }
        } else {
            let mut input_pressed = true;
            let node = match input {
                OSInput::Shift => {
                    self.input_shift = false;
                    None
                }
                OSInput::Digit(0) => Some(UnstructuredNode::Token(Token::Variable('x'))),

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
            self.root.insert(&mut self.nav_path, renderer, self.viewport.as_mut(), node);
        }

        None
    }
}

const MINIMUM_PAREN_HEIGHT: u64 = 16;

impl Renderer for ApplicationFrameworkInterface {
    fn size(&mut self, glyph: Glyph) -> Area {
        // Calculate an average character size
        let (text_character_width, text_character_height) = framework().display.string_size("0");
        let text_character_size = Area {
            width: text_character_width as u64,
            height: text_character_height as u64,
        };

        match glyph {
            Glyph::Cursor { height } => Area { height, width: 0 },
            Glyph::Placeholder => text_character_size,

            Glyph::Digit { .. } => text_character_size,
            Glyph::Variable { name } => {
                let (width, height) = framework().display.string_size(&name.to_string());
                Area {
                    width: width as u64,
                    height: height as u64,
                }
            },

            Glyph::Point => Area { width: text_character_size.width / 2, ..text_character_size },
            Glyph::Add => text_character_size,
            Glyph::Subtract => text_character_size,
            Glyph::Multiply => text_character_size,
            Glyph::Divide => text_character_size,
            Glyph::Fraction { inner_width } => Area { height: 1, width: inner_width },

            Glyph::LeftParenthesis { inner_height } => Area { width: 5, height: max(inner_height, MINIMUM_PAREN_HEIGHT) },
            Glyph::RightParenthesis { inner_height } => Area { width: 5, height: max(inner_height, MINIMUM_PAREN_HEIGHT) },

            Glyph::Sqrt { .. } => unimplemented!(),
        }
    }

    fn init(&mut self, _size: Area) {}

    fn draw(&mut self, glyph: ViewportGlyph) {
        // Apply padding
        let mut glyph = ViewportGlyph {
            point: glyph.point.dx(self.rbop_location_x).dy(self.rbop_location_y),
            ..glyph
        };

        match glyph.visibility {
            ViewportVisibility::Clipped { invisible, .. } if invisible => return,
            ViewportVisibility::Clipped { left_clip, right_clip, .. } => {
                // Re-align and shorten a left-clipped fraction line
                if let Glyph::Fraction { inner_width } = glyph.glyph.glyph {
                    if left_clip > 0 {
                        glyph.glyph = Glyph::Fraction {
                            inner_width: inner_width - left_clip
                        }.to_sized(self);
                        glyph.point.x = 0;
                    }
                }

                // Shorten a right-clipped fraction line
                // (The if-let binding is repeated to get a possibly updated inner_width)
                if let Glyph::Fraction { inner_width } = glyph.glyph.glyph {
                    if right_clip > 0 {
                        glyph.glyph = Glyph::Fraction {
                            inner_width: inner_width - right_clip
                        }.to_sized(self);
                    }
                }                
            }
            _ => (),
        }

        let point = glyph.point;

        match glyph.glyph.glyph {
            Glyph::Digit { number } => self.display.draw_char(point.x, point.y, (number + '0' as u8) as char),
            Glyph::Point => self.display.draw_char(point.x, point.y, '.'),
            Glyph::Variable { name } => self.display.draw_char(point.x, point.y, name),
            Glyph::Add => self.display.draw_char(point.x, point.y, '+'),
            Glyph::Subtract => self.display.draw_char(point.x, point.y, '-'),
            Glyph::Multiply => self.display.draw_char(point.x, point.y, '*'),
            Glyph::Divide => self.display.draw_char(point.x, point.y, '/'),
            
            Glyph::Fraction { inner_width } =>
                self.display.draw_line(point.x, point.y, point.x + inner_width as i64, point.y, Colour::WHITE),
            
            Glyph::Cursor { height } =>
                self.display.draw_line(point.x, point.y, point.x, point.y + height as i64, Colour::WHITE),

            Glyph::Placeholder => self.display.draw_rect(
                point.x + 4, point.y + 5, 6, 6, Colour::GREY, ShapeFill::Filled, 0
            ),

            Glyph::LeftParenthesis { inner_height } => {
                let inner_height = max(MINIMUM_PAREN_HEIGHT, inner_height) as i64;
                
                self.display.draw_line(point.x + 3, point.y, point.x + 3, point.y + 1, Colour::WHITE);
                self.display.draw_line(point.x + 2, point.y + 2, point.x + 2, point.y + 6, Colour::WHITE);

                self.display.draw_line(point.x + 1, point.y + 7, point.x + 1, point.y + inner_height - 8, Colour::WHITE);

                self.display.draw_line(point.x + 3, point.y + inner_height - 2, point.x + 3, point.y + inner_height - 1, Colour::WHITE);
                self.display.draw_line(point.x + 2, point.y + inner_height - 7, point.x + 2, point.y + inner_height - 3, Colour::WHITE);
            }
            Glyph::RightParenthesis { inner_height } => {
                let inner_height = max(MINIMUM_PAREN_HEIGHT, inner_height) as i64;
                
                self.display.draw_line(point.x + 1, point.y, point.x + 1, point.y + 1, Colour::WHITE);
                self.display.draw_line(point.x + 2, point.y + 2, point.x + 2, point.y + 6, Colour::WHITE);

                self.display.draw_line(point.x + 3, point.y + 7, point.x + 3, point.y + inner_height - 8, Colour::WHITE);

                self.display.draw_line(point.x + 1, point.y + inner_height - 2, point.x + 1, point.y + inner_height - 1, Colour::WHITE);
                self.display.draw_line(point.x + 2, point.y + inner_height - 7, point.x + 2, point.y + inner_height - 3, Colour::WHITE);
            }

            Glyph::Sqrt { .. } => unimplemented!(),
        }
    }
}
