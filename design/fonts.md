# Fonts

The Delta Pico uses a custom monochrome font format with transparency support, so that fonts can be
anti-aliased against their background. Key details:

- Each glyph can be up to 255x255
- 4 bits of alpha per pixel (0-15)
- Only ASCII characters are supported

## Glyph Format

Compared to the [16-bit colour bitmap format](bitmaps) used for images, this font format is more
more simple.

The first byte is the width of the glyph, and the second byte is the height of the glyph. After
this short header, every other byte is pixel data. Pixels are encoded with a top-left origin, in 
columns (not rows).

Every **nibble** (4 bits) after the header is the alpha of that pixel, i.e. how transparent is
should be when drawn. 0 means that pixel is fully transparent and won't be drawn at all, 15 (0xF)
means it'll be drawn fully opaque, and a value between the two will blend the font and background
colours.

If the glyph contains an odd number of pixels, so that the alpha nibbles do not fit evenly into the 
number of bytes, the final nibble will be ignored.

## Font Format

Fonts are 256-element arrays of pointers to ASCII glyphs: indexing the font array with a particular
ASCII value will return a pointer to the glyph data for that character.

If a font does not support a particular ASCII character, it should return `NULL` instead. These
characters will be omitted when rendering text.

## Text Rendering

The Delta Pico's text renderer is very naive. It does not apply any character spacing - spacing must
be included within the glyph. Any fully transparent pixels such as spacing will also be considered
when performing string length measurements, so you may wish to apply spacing evenly to both sides of
a glyph to ensure it looks correct when the OS centers text.

There is one special case when rendering: the newline character `\n` (ASCII dec 10, hex 0xA) will
never be rendered as a character, even in the font provides a glyph for it. Instead, the text cursor
will set the current X position to 0, and increase the Y position by the height of the glyph for `A`
(ASCII dec 65, hex 0x41) provided by the font.

This special case does not apply to any other control characters - they have no effect on the
cursor, and it is possible to provide glyphs for those which would be rendered.

## Compression

Glyphs are not currently compressed in any way, though this may be a good improvement for the
future. Run-length encoding is unlikely to yield much of a benefit for the characters themselves,
but it could be very beneficial for the large transparent areas around characters.

An easy improvement would be to allow each glyph to specify a "glyph size", "bitmap size", and
"bitmap offset". When drawn, the glyph would advance the cursor by its "glyph size", but the
transparency padding the bitmap can be removed, and instead only draw a bitmap within a smaller
bounding box defined by the "bitmap size" and "bitmap offset".
