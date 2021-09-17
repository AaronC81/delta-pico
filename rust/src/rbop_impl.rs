use alloc::{format, vec};
use rbop::{Token, UnstructuredNode, UnstructuredNodeList, nav::{MoveVerticalDirection, NavPath}, node::unstructured::{UnstructuredNodeRoot, MoveResult}, render::{Area, CalculatedPoint, Glyph, Renderer, Viewport, ViewportGlyph, ViewportVisibility}};
use crate::{debug, graphics::colour, interface::{ApplicationFrameworkInterface, ButtonInput, framework}, operating_system::os};

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

    pub fn input(&mut self, input: ButtonInput) -> Option<(MoveVerticalDirection, MoveResult)> {
        let renderer = framework();

        let node_to_insert = if !self.input_shift {
            match input {
                ButtonInput::None => None,
        
                ButtonInput::MoveLeft => {
                    self.root.move_left(&mut self.nav_path, renderer, self.viewport.as_mut());
                    None
                }
                ButtonInput::MoveRight => {
                    self.root.move_right(&mut self.nav_path, renderer, self.viewport.as_mut());
                    None
                }
                ButtonInput::MoveUp => {
                    return Some((
                        MoveVerticalDirection::Up,
                        self.root.move_up(&mut self.nav_path, renderer, self.viewport.as_mut())
                    ));
                }
                ButtonInput::MoveDown => {
                    return Some((
                        MoveVerticalDirection::Down,
                        self.root.move_down(&mut self.nav_path, renderer, self.viewport.as_mut())
                    ));
                }
                ButtonInput::Delete => {
                    self.root.delete(&mut self.nav_path, renderer, self.viewport.as_mut());
                    None
                }
        
                ButtonInput::Digit0 => Some(UnstructuredNode::Token(Token::Digit(0))),
                ButtonInput::Digit1 => Some(UnstructuredNode::Token(Token::Digit(1))),
                ButtonInput::Digit2 => Some(UnstructuredNode::Token(Token::Digit(2))),
                ButtonInput::Digit3 => Some(UnstructuredNode::Token(Token::Digit(3))),
                ButtonInput::Digit4 => Some(UnstructuredNode::Token(Token::Digit(4))),
                ButtonInput::Digit5 => Some(UnstructuredNode::Token(Token::Digit(5))),
                ButtonInput::Digit6 => Some(UnstructuredNode::Token(Token::Digit(6))),
                ButtonInput::Digit7 => Some(UnstructuredNode::Token(Token::Digit(7))),
                ButtonInput::Digit8 => Some(UnstructuredNode::Token(Token::Digit(8))),
                ButtonInput::Digit9 => Some(UnstructuredNode::Token(Token::Digit(9))),
        
                ButtonInput::Point => Some(UnstructuredNode::Token(Token::Point)),
                ButtonInput::LeftParen | ButtonInput::RightParen => Some(UnstructuredNode::Parentheses(
                    UnstructuredNodeList { items: vec![] },
                )),
        
                ButtonInput::Add => Some(UnstructuredNode::Token(Token::Add)),
                ButtonInput::Subtract => Some(UnstructuredNode::Token(Token::Subtract)),
                ButtonInput::Multiply => Some(UnstructuredNode::Token(Token::Multiply)),
                ButtonInput::Fraction => Some(UnstructuredNode::Fraction(
                    UnstructuredNodeList { items: vec![] },
                    UnstructuredNodeList { items: vec![] },
                )),
                ButtonInput::Power => Some(UnstructuredNode::Power(
                    UnstructuredNodeList { items: vec![] },
                )),

                ButtonInput::Exe => return None,
                ButtonInput::List => return None,
                ButtonInput::Shift => {
                    self.input_shift = true;
                    None
                }

                // Handled higher up
                ButtonInput::Menu => panic!("Unhandled MENU keypress"),
            }
        } else {
            let mut input_pressed = true;
            let node = match input {
                ButtonInput::Shift => {
                    self.input_shift = false;
                    None
                }
                ButtonInput::Digit0 => Some(UnstructuredNode::Token(Token::Variable('x'))),

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
        let text_character_size = Area { height: 8 * 2, width: 6 * 2 };

        match glyph {
            Glyph::Cursor { height } => Area { height, width: 0 },
            Glyph::Placeholder => text_character_size,

            Glyph::Digit { .. } => text_character_size,
            Glyph::Variable { .. } => text_character_size,

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

    fn init(&mut self, size: Area) {}

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
            Glyph::Digit { number } => (self.display.draw_char)(point.x, point.y, number + ('0' as u8)),
            Glyph::Point => (self.display.draw_char)(point.x, point.y, '.' as u8),
            Glyph::Variable { name } => (self.display.draw_char)(point.x, point.y, name as u8),
            Glyph::Add => (self.display.draw_char)(point.x, point.y, '+' as u8),
            Glyph::Subtract => (self.display.draw_char)(point.x, point.y, '-' as u8),
            Glyph::Multiply => (self.display.draw_char)(point.x, point.y, '*' as u8),
            Glyph::Divide => (self.display.draw_char)(point.x, point.y, '/' as u8),
            
            Glyph::Fraction { inner_width } =>
            (self.display.draw_line)(point.x, point.y, point.x + inner_width as i64, point.y, 0xFFFF),
            
            Glyph::Cursor { height } =>
                (self.display.draw_line)(point.x, point.y, point.x, point.y + height as i64, 0xFFFF),

            Glyph::Placeholder => (self.display.draw_rect)(
                point.x + 4, point.y + 5, 6, 6, colour::GREY, true, 0
            ),

            Glyph::LeftParenthesis { inner_height } => {
                let inner_height = max(MINIMUM_PAREN_HEIGHT, inner_height) as i64;
                
                (self.display.draw_line)(point.x + 3, point.y, point.x + 3, point.y + 1, 0xFFFF);
                (self.display.draw_line)(point.x + 2, point.y + 2, point.x + 2, point.y + 6, 0xFFFF);

                (self.display.draw_line)(point.x + 1, point.y + 7, point.x + 1, point.y + inner_height - 8, 0xFFFF);

                (self.display.draw_line)(point.x + 3, point.y + inner_height - 2, point.x + 3, point.y + inner_height - 1, 0xFFFF);
                (self.display.draw_line)(point.x + 2, point.y + inner_height - 7, point.x + 2, point.y + inner_height - 3, 0xFFFF);
            }
            Glyph::RightParenthesis { inner_height } => {
                let inner_height = max(MINIMUM_PAREN_HEIGHT, inner_height) as i64;
                
                (self.display.draw_line)(point.x + 1, point.y, point.x + 1, point.y + 1, 0xFFFF);
                (self.display.draw_line)(point.x + 2, point.y + 2, point.x + 2, point.y + 6, 0xFFFF);

                (self.display.draw_line)(point.x + 3, point.y + 7, point.x + 3, point.y + inner_height - 8, 0xFFFF);

                (self.display.draw_line)(point.x + 1, point.y + inner_height - 2, point.x + 1, point.y + inner_height - 1, 0xFFFF);
                (self.display.draw_line)(point.x + 2, point.y + inner_height - 7, point.x + 2, point.y + inner_height - 3, 0xFFFF);
            }

            Glyph::Sqrt { .. } => unimplemented!(),
        }
    }
}
