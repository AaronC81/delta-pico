use core::{fmt::Debug, convert::TryInto};

pub trait AsciiFont: Debug {
    fn char_data(&self, c: u8) -> Option<AsciiFontGlyph<'static>>;

    fn string_size(&self, string: &str) -> (i16, i16) {
        // Use 'A' as the height of a line
        let line_height = self.char_data(b'A').unwrap().height;

        let mut current_line_width= 0u16;
        let mut longest_line_width = 0u16;
        let mut height = line_height;

        for c in string.chars() {
            if c != '\n' {
                if let Some(glyph) = c.try_into().ok().and_then(|c| self.char_data(c)) {
                    // Update line width
                    current_line_width += glyph.width as u16;
                    if current_line_width > longest_line_width {
                        longest_line_width = current_line_width;
                    }
                }
            } else {
                height += line_height;
                current_line_width = 0;
            }
        }

        (longest_line_width as i16, height as i16)
    }
}

#[derive(Debug)]
pub struct AsciiFontGlyph<'a> {
    pub width: u8,
    pub height: u8,
    pub data: &'a [u8],
}
