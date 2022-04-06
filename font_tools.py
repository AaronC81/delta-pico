import os
import tempfile
import sys

def write_font_glyphs(font_path, size, glyphs_path) -> str:
    import fontforge

    font = fontforge.open(font_path)

    # For each ASCII glyph...
    for ascii in range(128):
        # Use FontForge to render it as an image to a temporary path (if the glyph exists)
        glyph = None
        try:
            glyph = font[ascii]
        except TypeError:
            # FontForge throws this if a glyph is missing
            continue

        glyph_image_file_path = os.path.join(glyphs_path, f"glyph_{ascii}.png")
        glyph.export(glyph_image_file_path, pixelsize=size)

def generate_font_source(name, glyphs_path) -> str:
    from PIL import Image

    # Load the image with PIL and iterate over its pixels to build up a Rust array
    result = f"#[derive(PartialEq, Eq, Debug, Copy, Clone)] pub struct {name};\n\nimpl {name} {{\n\n"
    valid_ids = []
    for glyph in sorted(os.listdir(glyphs_path)):
        # Validate name (so we don't try to process a .DS_Store or something)
        if not (glyph.startswith("glyph_") and glyph.endswith(".png")):
            continue

        # The Rust name is {font name}_{glyph ASCII ID} - the [6:-4] chops off the glyph_ and .png
        glyph_id = int(glyph[6:-4])
        valid_ids.append(glyph_id)
        glyph_c_name = f"{name}_{glyph_id}".upper()

        glyph_image_file_path = os.path.join(glyphs_path, glyph)
        glyph_image = Image.open(glyph_image_file_path)
        result += f"const {glyph_c_name}: [u8; Self::{glyph_c_name}_LEN] = [{glyph_image.width}, {glyph_image.height}"
        buffer = None
        length = 2
        for x in range(glyph_image.width):
            for y in range(glyph_image.height):
                # All channels are the same, so just check the red one of each pixel
                value = glyph_image.getpixel((x, y))

                # To reduce 8 bits to 4 bits, get the 4 most significant bits
                value = (0xF0 & value) >> 4

                if buffer is None:

                    buffer = f"0x{hex(value)[-1]}"
                else:
                    buffer += hex(value)[-1]
                    result += f", {buffer}"
                    buffer = None
                    length += 1

        # In case of an odd number of pixels in a glyph, check the buffer and pop it
        if buffer is not None:
            result += f", 0x{hex(value)[-1]}0"
            length += 1
        
        result += "];\n"

        result += f"const {glyph_c_name}_LEN: usize = {length};\n"

    # Finally, generate a table to look up these glyph bitmaps
    result += f"}}\n\nimpl crate::graphics::AsciiFont for {name} {{\n"
    result += f"fn char_data(&self, c: u8) -> Option<&'static [u8]> {{\n"
    result +=  "    match c {\n"

    for glyph_id in range(256):
        if glyph_id in valid_ids:
            glyph_c_name = f"{name}_{glyph_id}".upper()
            result += f"        {glyph_id} => Some(&Self::{glyph_c_name}[..]),\n"
        else:
            result += f"        {glyph_id} => None,\n"

    result +=  "    }\n"
    result +=  "}\n}\n"

    return result

if __name__ == "__main__":
    # Exists because we need an easy way of running this in FontForge's interpreter
    # Usage:
    #   ffpython font_tools.py glyphs /path/to/font.ttf font_size /path/to/glyphs/
    if sys.argv[1] == "glyphs":
        write_font_glyphs(sys.argv[2], int(sys.argv[3]), sys.argv[4])
    else:
        raise KeyError(f"Unknown command {sys.argv[1]}")
