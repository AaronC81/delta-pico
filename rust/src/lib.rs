#![no_std]
#![feature(default_alloc_error_handler)]

extern crate alloc;

mod c_allocator;

use core::panic::PanicInfo;
use alloc::{boxed::Box, format, string::{String, ToString}, vec::Vec, vec};
use rbop::{Token, UnstructuredNode, UnstructuredNodeList, nav::NavPath, node::unstructured::{UnstructuredNodeRoot, Upgradable}, render::{Area, CalculatedPoint, Glyph, Renderer, Viewport, ViewportGlyph, ViewportVisibility}};
use c_allocator::CAllocator;
use rust_decimal::prelude::ToPrimitive;

#[global_allocator]
static ALLOCATOR: CAllocator = CAllocator;

pub struct RbopContext {
    root: UnstructuredNodeRoot,
    nav_path: NavPath,
    viewport: Option<Viewport>,
}

static mut FRAMEWORK: *mut ApplicationFrameworkInterface = 0 as *mut _;
pub fn framework() -> &'static mut ApplicationFrameworkInterface {
    unsafe {
        FRAMEWORK.as_mut().unwrap()
    }
}

#[no_mangle]
pub extern "C" fn delta_pico_set_framework(fw: *mut ApplicationFrameworkInterface) {
    unsafe {
        FRAMEWORK = fw;
    }
}

#[repr(C)]
pub struct ApplicationFrameworkInterface {
    panic_handler: extern "C" fn(*const u8) -> (),
    debug_handler: extern "C" fn(*const u8) -> (),
    display: DisplayInterface,
    buttons: ButtonsInterface,
}

#[repr(C)]
pub struct DisplayInterface {
    fill_screen: extern "C" fn(c: u16),
    draw_char: extern "C" fn(x: i64, y: i64, character: u8),
    draw_line: extern "C" fn(x1: i64, y1: i64, x2: i64, y2: i64, c: u16),
    print: extern "C" fn(s: *const u8),
    set_cursor: extern "C" fn(x: i64, y: i64),
    draw: extern "C" fn(),
}

#[repr(C)]
#[derive(PartialEq, Eq)]
pub enum ButtonEvent {
    Press,
    Release,
}

#[repr(C)]
pub struct ButtonsInterface {
    poll_input_event: extern "C" fn(input: *mut RbopInput, event: *mut ButtonEvent) -> bool,
}

impl ButtonsInterface {
    fn poll_press(&self) -> Option<RbopInput> {
        // Garbage default values
        let mut input: RbopInput = RbopInput::None;
        let mut event: ButtonEvent = ButtonEvent::Release;

        if (self.poll_input_event)(&mut input as *mut _, &mut event as *mut _) && event == ButtonEvent::Press {
            Some(input)
        } else {
            None
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

    fn draw(&mut self, mut glyph: ViewportGlyph) {
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

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let message = format!("{}", info);
    let mut message_bytes = message.as_bytes().iter().cloned().collect::<Vec<_>>();
    message_bytes.push(0);

    (framework().panic_handler)(message_bytes.as_ptr());
    loop {}
}

fn debug(info: String) {
    let mut message_bytes = info.as_bytes().iter().cloned().collect::<Vec<_>>();
    message_bytes.push(0);

    (framework().debug_handler)(message_bytes.as_ptr());
}

#[no_mangle]
pub extern "C" fn delta_pico_main() {
    debug("Rust main!".into());

    // Set up context
    let mut rbop_ctx = RbopContext {
        root: UnstructuredNodeRoot { root: UnstructuredNodeList { items: vec![] } },
        nav_path: NavPath::new(vec![0]),
        viewport: Some(Viewport::new(Area::new(220, 300))), // TODO: use C constants here
    };

    loop {
        // Draw
        framework().draw_all(
            &rbop_ctx.root, 
            Some(&mut rbop_ctx.nav_path.to_navigator()),
            rbop_ctx.viewport.as_ref(),
        );

        // Evaluate
        let result = if let Ok(structured) = rbop_ctx.root.upgrade() {
            if let Ok(evaluation_result) = structured.evaluate() {
                Some(evaluation_result)
            } else {
                None
            }
        } else {
            None
        };

        // Write result
        if let Some(result) = result {
            let result_str = result.to_string();
            let mut result_chars = result_str.as_bytes().to_vec();
            result_chars.push(0);

            (framework().display.set_cursor)(0, 300 - 30);
            (framework().display.print)(result_chars.as_ptr());
        }

        // Push to screen
        (framework().display.draw)();

        // Poll for input
        if let Some(input) = framework().buttons.poll_press() {
            rbop_input(&mut rbop_ctx as *mut _, input)
        }
    }
}

/// All possible user inputs.
#[repr(C)]
pub enum RbopInput {
    None,

    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Delete,

    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,

    Point,

    Add,
    Subtract,
    Multiply,
    Fraction,
}

/// Manipulates an rbop context based on an input.
#[no_mangle]
pub extern "C" fn rbop_input(ctx: *mut RbopContext, input: RbopInput) {
    let ctx = unsafe { ctx.as_mut().unwrap() };
    let renderer = framework();

    let node_to_insert = match input {
        RbopInput::None => None,

        RbopInput::MoveLeft => {
            ctx.root.move_left(&mut ctx.nav_path, renderer, ctx.viewport.as_mut());
            None
        }
        RbopInput::MoveRight => {
            ctx.root.move_right(&mut ctx.nav_path, renderer, ctx.viewport.as_mut());
            None
        }
        RbopInput::MoveUp => {
            ctx.root.move_up(&mut ctx.nav_path, renderer, ctx.viewport.as_mut());
            None
        }
        RbopInput::MoveDown => {
            ctx.root.move_down(&mut ctx.nav_path, renderer, ctx.viewport.as_mut());
            None
        }
        RbopInput::Delete => {
            ctx.root.delete(&mut ctx.nav_path, renderer, ctx.viewport.as_mut());
            None
        }

        RbopInput::Digit0 => Some(UnstructuredNode::Token(Token::Digit(0))),
        RbopInput::Digit1 => Some(UnstructuredNode::Token(Token::Digit(1))),
        RbopInput::Digit2 => Some(UnstructuredNode::Token(Token::Digit(2))),
        RbopInput::Digit3 => Some(UnstructuredNode::Token(Token::Digit(3))),
        RbopInput::Digit4 => Some(UnstructuredNode::Token(Token::Digit(4))),
        RbopInput::Digit5 => Some(UnstructuredNode::Token(Token::Digit(5))),
        RbopInput::Digit6 => Some(UnstructuredNode::Token(Token::Digit(6))),
        RbopInput::Digit7 => Some(UnstructuredNode::Token(Token::Digit(7))),
        RbopInput::Digit8 => Some(UnstructuredNode::Token(Token::Digit(8))),
        RbopInput::Digit9 => Some(UnstructuredNode::Token(Token::Digit(9))),

        RbopInput::Point => Some(UnstructuredNode::Token(Token::Point)),

        RbopInput::Add => Some(UnstructuredNode::Token(Token::Add)),
        RbopInput::Subtract => Some(UnstructuredNode::Token(Token::Subtract)),
        RbopInput::Multiply => Some(UnstructuredNode::Token(Token::Multiply)),
        RbopInput::Fraction => Some(UnstructuredNode::Fraction(
            UnstructuredNodeList { items: vec![] },
            UnstructuredNodeList { items: vec![] },
        )),
    };

    if let Some(node) = node_to_insert {
        ctx.root.insert(&mut ctx.nav_path, renderer, ctx.viewport.as_mut(), node);
    }
}
