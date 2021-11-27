# Bitmaps

The Delta Pico uses a partially-run-length-encoded bitmap format which is extremely easy to encode
and decode. The Delta Pico has a relatively slow frame time even in the best scenarios, so decoding
speed is an important factor. 

Overview of what's supported:

- Width or height up to 2^16 (65536)
- RGB565 colour, minus any two colours (used as special markers within the image)
- Transparency - only fully transparent or fully opaque, no partial transparency is supported

## Encoding Format

A bitmap is encoded as an array of 16-bit integers. The first four items of the array are metadata:

- Image width
- Image height
- A 16-bit marker to represent transparency
- A 16-bit marker to represent the start of run-length encoding

After these elements, pixel data follows. Pixels are encoded with a top-left origin, in columns (not
rows).

Each pixel datum may be one of...

### A 16-bit colour

The colour is simply drawn as encoded.

### The transparency marker

Drawing this pixel will be skipped completely, leaving what was already in that position on the
screen, resulting in a transparent pixel.

### The run-length encoding marker

If the datum is the run-length encoding marker, more elements must be read to gather all required
data. The element following the marker is a count: how many consecutive pixels use the same colour.
The element following the count is the colour to be repeated, which is permitted to be the
transparency marker.

To simplify decoding, run-length pixel encodings may not wrap around the end of a column. Besides
this restriction, run-length encoding is always valid in place of repeated 16-bit colour data and
vice versa: for a colour `A`, pixel encodings `A` and `Y 1 A` are interchangable, though the former
encoding is more space-efficient.

## Example

Take the following 3x5 example image, with 5 unique 16-bit colours represented by letters `A` to
`E`:

```
ABD
ACD
A E
ADE
ADE
```

As with many images in this encoding, there are multiple valid encodings, but the smallest one is
as follows, with array elements visualised as being separated by whitespace for clarity:

```
3 5 X Y 
Y 5 A
B C X D D
D D E E E
```

`3 5 X Y` is the metadata. This image has width 3, height 5, and defines unused colours `X` and `Y`
as transparency and run-length markers respectively.

`Y 5 A` encodes the first column, using a run-length encoding of length 5 repeating the colour `A`.
`A A A A A` would have been an equally valid encoding, but that is longer.

`B C X D D` encodes the second column. The `B` and `C` colours only appear for a single pixel, so
run-length encoding would be a waste of space, but `Y 1 B` and `Y 1 C` would be valid. `X` encodes
the transparent pixel in the middle of the column. Finally, `D D` is used - while the `D` pixels
do continue onto the next column, run-length encoding across columns is not allowed, so `D D` is the
most space-efficient encoding.

`D D E E E` encodes the third and final column, simply encoding all pixels individually. For the `E`
pixels, `Y 3 E` would take up identical space, but requires more decoding logic, so `E E E` is the
preferred form.
