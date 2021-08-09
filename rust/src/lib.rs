#![no_std]
#![feature(default_alloc_error_handler)]

extern crate alloc;

mod c_allocator;

use core::panic::PanicInfo;
use alloc::{boxed::Box, format, string::String, vec::Vec, vec};
use rbop::{Token, UnstructuredNode, UnstructuredNodeList, nav::NavPath, node::unstructured::{UnstructuredNodeRoot, Upgradable}, render::{Area, CalculatedPoint, Glyph, Renderer, Viewport, ViewportGlyph, ViewportVisibility}};
use c_allocator::CAllocator;
use rust_decimal::prelude::ToPrimitive;

#[global_allocator]
static ALLOCATOR: CAllocator = CAllocator;

pub struct RbopContext {
    root: UnstructuredNodeRoot,
    nav_path: NavPath,
    renderer: *mut RbopRendererInterface,
    viewport: Option<Viewport>,
}

#[repr(C)]
pub struct RbopRendererInterface {
    clear: extern "C" fn() -> (),
    draw_char: extern "C" fn(x: i64, y: i64, character: u8),
    draw_line: extern "C" fn(x1: i64, y1: i64, x2: i64, y2: i64),
}

impl Renderer for RbopRendererInterface {
    fn size(&mut self, glyph: Glyph) -> Area {
        let text_character_size = Area { height: 8 * 3, width: 6 * 3 };

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
        (self.clear)()
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
            Glyph::Digit { number } => (self.draw_char)(point.x, point.y, number + ('0' as u8)),
            Glyph::Point => (self.draw_char)(point.x, point.y, '.' as u8),
            Glyph::Add => (self.draw_char)(point.x, point.y, '+' as u8),
            Glyph::Subtract => (self.draw_char)(point.x, point.y, '-' as u8),
            Glyph::Multiply => (self.draw_char)(point.x, point.y, '*' as u8),
            Glyph::Divide => (self.draw_char)(point.x, point.y, '/' as u8),

            Glyph::Fraction { inner_width } =>
                (self.draw_line)(point.x, point.y, point.x + inner_width as i64, point.y),

            Glyph::Cursor { height } =>
                (self.draw_line)(point.x, point.y, point.x, point.y + height as i64),

            _ => todo!(),
        }
    }
}

static mut PANIC_HANDLER_FN: Option<extern "C" fn(*const u8) -> ()> = None;
static mut DEBUG_HANDLER_FN: Option<extern "C" fn(*const u8) -> ()> = None;

/// Sets a function to be called when a Rust panic occurs.
#[no_mangle]
pub extern "C" fn rbop_set_panic_handler(func: extern "C" fn(*const u8) -> ()) {
    unsafe {
        PANIC_HANDLER_FN = Some(func);
    }
}

/// Sets a function to be called when this library wishes to print debugging information.
#[no_mangle]
pub extern "C" fn rbop_set_debug_handler(func: extern "C" fn(*const u8) -> ()) {
    unsafe {
        DEBUG_HANDLER_FN = Some(func);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        let message = format!("{}", info);
        let mut message_bytes = message.as_bytes().iter().cloned().collect::<Vec<_>>();
        message_bytes.push(0);

        (PANIC_HANDLER_FN.unwrap())(message_bytes.as_ptr());
        loop {}
    }
}

fn debug(info: String) {
    unsafe {
        let mut message_bytes = info.as_bytes().iter().cloned().collect::<Vec<_>>();
        message_bytes.push(0);

        (DEBUG_HANDLER_FN.unwrap())(message_bytes.as_ptr());
    }
}

/// Allocates and returns a new rbop context.
#[no_mangle]
pub extern "C" fn rbop_new(renderer: *mut RbopRendererInterface) -> *mut RbopContext {
    Box::into_raw(Box::new(RbopContext {
        root: UnstructuredNodeRoot { root: UnstructuredNodeList { items: vec![] } },
        nav_path: NavPath::new(vec![0]),
        renderer,
        viewport: None,
    }))
}

/// Sets up a new viewport for the given rbop context.
#[no_mangle]
pub extern "C" fn rbop_set_viewport(ctx: *mut RbopContext, width: u64, height: u64) {
    unsafe {
        ctx.as_mut().unwrap().viewport = Some(Viewport::new(Area::new(width, height)));
    }
}

/// Frees an allocated rbop context.
#[no_mangle]
pub extern "C" fn rbop_free(ctx: *mut RbopContext) {
    // From the FFI Omnibus
    // To "free" a Box which we grabbed as a raw pointer, we just need to get the Box back
    // Since the Box then goes out-of-scope, it is dropped and the memory is freed
    // Thanks, Rust!
    unsafe { Box::from_raw(ctx); }
}

/// Renders an rbop context onto the screen.
#[no_mangle]
pub extern "C" fn rbop_render(ctx: *mut RbopContext) {
    unsafe {
        let ctx = ctx.as_mut().unwrap();
        ctx.renderer.as_mut().unwrap().draw_all(
            &ctx.root, 
            Some(&mut ctx.nav_path.to_navigator()),
            ctx.viewport.as_ref(),
        );
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
    let renderer = unsafe { ctx.renderer.as_mut().unwrap()  };

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

/// Evaluates an rbop context.
#[no_mangle]
pub extern "C" fn rbop_evaluate(ctx: *mut RbopContext, result: *mut f64) -> bool {
    let ctx = unsafe { ctx.as_mut().unwrap() };
    if let Ok(structured) = ctx.root.upgrade() {
        if let Ok(evaluation_result) = structured.evaluate() {
            unsafe { *result = evaluation_result.to_f64().unwrap(); }
            true
        } else {
            false
        }
    } else {
        false
    }
}
