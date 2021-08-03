#![no_std]
#![feature(default_alloc_error_handler)]

extern crate alloc;

mod c_allocator;

use core::panic::PanicInfo;
use alloc::{boxed::Box, format, vec::Vec, vec};
use rbop::{Token, UnstructuredNode, UnstructuredNodeList, nav::NavPath, node::unstructured::UnstructuredNodeRoot, render::{Area, CalculatedPoint, Glyph, Renderer}};
use c_allocator::CAllocator;

#[global_allocator]
static ALLOCATOR: CAllocator = CAllocator;

pub struct RbopContext {
    root: UnstructuredNodeRoot,
    nav_path: NavPath,
}

#[repr(C)]
pub struct RbopRendererInterface {
    clear: extern "C" fn() -> (),
    draw_char: extern "C" fn(x: u64, y: u64, character: u8),
    draw_line: extern "C" fn(x1: u64, y1: u64, x2: u64, y2: u64),
}

impl Renderer for RbopRendererInterface {
    fn size(&mut self, glyph: Glyph) -> Area {
        let text_character_size = Area { height: 8, width: 6 };

        match glyph {
            Glyph::Cursor { height } => Area { height, width: 1 },
            Glyph::Digit { .. } => text_character_size,
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

    fn draw(&mut self, glyph: Glyph, point: CalculatedPoint) {
        match glyph {
            Glyph::Digit { number } => (self.draw_char)(point.x, point.y, number + ('0' as u8)),
            Glyph::Add => (self.draw_char)(point.x, point.y, '+' as u8),
            Glyph::Subtract => (self.draw_char)(point.x, point.y, '-' as u8),
            Glyph::Multiply => (self.draw_char)(point.x, point.y, '*' as u8),
            Glyph::Divide => (self.draw_char)(point.x, point.y, '/' as u8),

            Glyph::Fraction { inner_width } =>
                (self.draw_line)(point.x, point.y, point.x + inner_width, point.y),

            Glyph::Cursor { height } =>
                (self.draw_line)(point.x, point.y, point.x, point.y + height),

            _ => todo!(),
        }
    }
}

static mut PANIC_HANDLER_FN: Option<extern "C" fn(*const u8) -> ()> = None;

/// Sets a function to be called when a Rust panic occurs.
#[no_mangle]
pub extern "C" fn rbop_set_panic_handler(func: extern "C" fn(*const u8) -> ()) {
    unsafe {
        PANIC_HANDLER_FN = Some(func);
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

/// Allocates and returns a new rbop context.
#[no_mangle]
pub extern "C" fn rbop_new() -> *mut RbopContext {
    Box::into_raw(Box::new(RbopContext {
        root: UnstructuredNodeRoot { root: UnstructuredNodeList { items: vec![] } },
        nav_path: NavPath::new(vec![0]),
    }))
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

#[no_mangle]
pub extern "C" fn rbop_foo(ctx: *mut RbopContext) {
    unsafe {
        let ctx = ctx.as_mut().unwrap();
        ctx.root.insert(&mut ctx.nav_path, UnstructuredNode::Token(Token::Digit(3)));
    }
}

/// Renders an rbop context onto the screen.
#[no_mangle]
pub extern "C" fn rbop_render(ctx: *mut RbopContext, renderer: *mut RbopRendererInterface) {
    unsafe {
        let ctx = ctx.as_mut().unwrap();
        renderer.as_mut().unwrap().draw_all(&ctx.root, Some(&mut ctx.nav_path.to_navigator()));
    }
}
