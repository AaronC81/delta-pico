use alloc::{format, vec};
use rbop::{Token, UnstructuredNode, UnstructuredNodeList, nav::NavPath, node::unstructured::{UnstructuredNodeRoot}, render::{Area, Glyph, Renderer, Viewport, ViewportGlyph, ViewportVisibility}};
use crate::{debug, interface::{ApplicationFrameworkInterface, ButtonInput, framework}};

pub const PADDING: u64 = 10;

pub struct RbopContext {
    pub root: UnstructuredNodeRoot,
    pub nav_path: NavPath,
    pub viewport: Option<Viewport>,
}

impl RbopContext {
    pub fn input(&mut self, input: ButtonInput) {
        let renderer = framework();

        let node_to_insert = match input {
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
                self.root.move_up(&mut self.nav_path, renderer, self.viewport.as_mut());
                None
            }
            ButtonInput::MoveDown => {
                self.root.move_down(&mut self.nav_path, renderer, self.viewport.as_mut());
                None
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
    
            ButtonInput::Add => Some(UnstructuredNode::Token(Token::Add)),
            ButtonInput::Subtract => Some(UnstructuredNode::Token(Token::Subtract)),
            ButtonInput::Multiply => Some(UnstructuredNode::Token(Token::Multiply)),
            ButtonInput::Fraction => Some(UnstructuredNode::Fraction(
                UnstructuredNodeList { items: vec![] },
                UnstructuredNodeList { items: vec![] },
            )),

            ButtonInput::Exe => return,

            // Handled higher up
            ButtonInput::Menu => panic!("Unhandled MENU keypress"),
        };
    
        if let Some(node) = node_to_insert {
            self.root.insert(&mut self.nav_path, renderer, self.viewport.as_mut(), node);
        }    
    }
}

impl Renderer for ApplicationFrameworkInterface {
    fn size(&mut self, glyph: Glyph) -> Area {
        let text_character_size = Area { height: 8 * 2, width: 6 * 2 };

        match glyph {
            Glyph::Cursor { height } => Area { height, width: 1 },
            Glyph::Digit { .. } => text_character_size,
            Glyph::Point => text_character_size,
            Glyph::Add => text_character_size,
            Glyph::Subtract => text_character_size,
            Glyph::Multiply => text_character_size,
            Glyph::Divide => text_character_size,
            Glyph::Fraction { inner_width } => Area { height: 1, width: inner_width },

            _ => unimplemented!(),
        }
    }

    fn init(&mut self, size: Area) {
        (self.display.fill_screen)(0);
    }

    fn draw(&mut self, glyph: ViewportGlyph) {
        // Apply padding
        let mut glyph = ViewportGlyph {
            point: glyph.point.dx(PADDING as i64).dy(PADDING as i64),
            ..glyph
        };

        debug(format!("{:?}", glyph));

        match glyph.visibility {
            ViewportVisibility::Clipped { invisible, .. } if invisible => return,
            ViewportVisibility::Clipped { left_clip, right_clip, .. } => {
                // Re-align and shorten a left-clipped fraction line
                if let Glyph::Fraction { inner_width } = glyph.glyph {
                    if left_clip > 0 {
                        glyph.glyph = Glyph::Fraction {
                            inner_width: inner_width - left_clip
                        };
                        glyph.point.x = 0;
                    }
                }

                // Shorten a right-clipped fraction line
                // (The if-let binding is repeated to get a possibly updated inner_width)
                if let Glyph::Fraction { inner_width } = glyph.glyph {
                    if right_clip > 0 {
                        glyph.glyph = Glyph::Fraction {
                            inner_width: inner_width - right_clip
                        };
                    }
                }                
            }
            _ => (),
        }

        let point = glyph.point;

        match glyph.glyph {
            Glyph::Digit { number } => (self.display.draw_char)(point.x, point.y, number + ('0' as u8)),
            Glyph::Point => (self.display.draw_char)(point.x, point.y, '.' as u8),
            Glyph::Add => (self.display.draw_char)(point.x, point.y, '+' as u8),
            Glyph::Subtract => (self.display.draw_char)(point.x, point.y, '-' as u8),
            Glyph::Multiply => (self.display.draw_char)(point.x, point.y, '*' as u8),
            Glyph::Divide => (self.display.draw_char)(point.x, point.y, '/' as u8),

            Glyph::Fraction { inner_width } =>
                (self.display.draw_line)(point.x, point.y, point.x + inner_width as i64, point.y, 0xFFFF),

            Glyph::Cursor { height } =>
                (self.display.draw_line)(point.x, point.y, point.x, point.y + height as i64, 0xFFFF),

            _ => todo!(),
        }
    }
}
