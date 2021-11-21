use alloc::{string::{String, ToString}, vec, vec::Vec};

#[repr(C)]
pub struct DisplayInterface {
    pub width: u64,
    pub height: u64,

    pub new_sprite: extern "C" fn(width: i16, height: i16) -> *mut u8,
    pub free_sprite: extern "C" fn(*mut u8),
    pub switch_to_sprite: extern "C" fn(*mut u8),
    pub switch_to_screen: extern "C" fn(),

    pub fill_screen: extern "C" fn(c: u16),
    pub draw_char: extern "C" fn(x: i64, y: i64, character: u8),
    pub draw_line: extern "C" fn(x1: i64, y1: i64, x2: i64, y2: i64, c: u16),
    pub draw_rect: extern "C" fn(x1: i64, y1: i64, w: i64, h: i64, c: u16, fill: bool, radius: u16),
    pub draw_sprite: extern "C" fn(x: i64, y: i64, sprite: *mut u8),
    pub draw_bitmap: extern "C" fn(x: i64, y: i64, name: *const u8),

    pub print: extern "C" fn(s: *const u8),
    pub set_cursor: extern "C" fn(x: i64, y: i64),
    pub get_cursor: extern "C" fn(x: *mut i64, y: *mut i64),

    pub draw: extern "C" fn(),
}

impl DisplayInterface {
    pub fn print(&self, s: impl Into<String>) {
        let mut bytes = s.into().as_bytes().to_vec();
        bytes.push(0);
        (self.print)(bytes.as_ptr())
    }

    pub fn print_at(&self, x: i64, y: i64, s: impl Into<String>) {
        (self.set_cursor)(x, y);
        self.print(s);
    }

    pub fn print_centred(&self, x: i64, y: i64, w: i64, s: impl Into<String>) {
        let s = s.into();
        let (text_width, _) = self.string_size(&s);

        let x_offset = (w - text_width) / 2;
        self.print_at(x + x_offset, y, s);
    }

    pub fn get_cursor(&self) -> (i64, i64) {
        let mut x: i64 = 0;
        let mut y: i64 = 0;

        (self.get_cursor)(&mut x as *mut _, &mut y as *mut _);
        (x, y)
    }

    pub fn string_size(&self, string: impl Into<String>) -> (i64, i64) {
        // Won't work for strings with newlines

        // Draw the string off the screen and see how much the cursor moved
        // HACK: This draws to a buffer, right? Could we be overwriting some random memory by
        // writing out of bounds??
        (self.set_cursor)(0, self.height as i64 + 100);
        self.print(string);
        let (x, _) = self.get_cursor();
        self.print("\n");
        let (_, y) = self.get_cursor();
        (x, y - (self.height as i64 + 100))
    }

    pub fn wrap_text(&self, string: impl Into<String>, width: i64) -> (Vec<String>, i64, i64) {
        // All characters are assumed to have the same height

        let mut x = 0;
        let mut y = 0;
        let mut lines: Vec<String> = vec!["".into()];
        let char_height = self.string_size("A").1;

        for word in Into::<String>::into(string).split_whitespace() {
            let (this_char_x, this_char_y) = self.string_size(word.to_string());
            x += this_char_x;
            if x > width {
                lines.push("".into());
                x = this_char_x;
                y += this_char_y;
            }

            lines.last_mut().unwrap().push_str(word);
            lines.last_mut().unwrap().push(' ');
        }

        // Factor in width of last line into height
        (lines, char_height, y + char_height)
    }

    pub fn draw_bitmap(&self, x: i64, y: i64, s: impl Into<String>) {
        let mut bytes = s.into().as_bytes().to_vec();
        bytes.push(0);
        (self.draw_bitmap)(x, y, bytes.as_ptr());
    }
}

