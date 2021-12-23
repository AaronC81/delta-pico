#include "ili9341.hpp"

#include <string.h>

void ILI9341Sprite::allocate() {
    data = new uint16_t[width * height];
}

void ILI9341Sprite::free() {
    delete data;
    delete this;
}

void ILI9341Sprite::fill(uint16_t colour) {
    // If the colour has the same upper and lower byte, we can use an optimised version of this
    // function instead
    if ((colour & 0xFF00 >> 8) == (colour & 0xFF)) {
        fill_fast((uint8_t)(colour & 0xFF));
        return;
    }

    draw_rect(0, 0, width, height, 0, true, colour);
}

void ILI9341Sprite::fill_fast(uint8_t half_colour) {
    for (uint16_t y = 0; y < height; y++) {
        memset(data + y * width, half_colour, width * 2);
    }
}

void ILI9341Sprite::draw_rect(uint16_t x, uint16_t y, uint16_t w, uint16_t h, uint16_t radius, bool filled, uint16_t colour) {
    // TODO: radius is ignored

    // This is a very frequently called function, so we want it to be as optimised as possible!
    // So we try to skip out unnecessary checks within the loop, and effectively duplicate the code
    // of the function based on what parameters we have.
    // Try to reduce code repetition with defines, but I'm not sure it helps :P

    #define RECT_LOOP for (uint16_t ix = 0; ix < w; ix++) { for (uint16_t iy = 0; iy < h; iy++) {
    #define RECT_END } }
    #define RECT_DRAW { draw_pixel(x + ix, y + iy, colour); }

    if (filled) {
        // Just draw everything as-is
        RECT_LOOP RECT_DRAW RECT_END
    } else {
        // Only draw pixels around the edges
        RECT_LOOP
            if (ix == 0 || iy == 0 || ix == w - 1 || iy == h - 1) RECT_DRAW
        RECT_END
    }

    #undef RECT_LOOP
    #undef RECT_END
    #undef RECT_DRAW
}

void ILI9341Sprite::draw_line(uint16_t x1, uint16_t y1, uint16_t x2, uint16_t y2, uint16_t colour) {
    // We expect the 1s to be lower than the 2s - if not, swap them
    if (x1 > x2) {
        int tmp = x2;
        x1 = x2;
        x2 = x1;
    }
    if (y1 > y2) {
        int tmp = y2;
        y1 = y2;
        y2 = y1;
    }

    // Only horizontal and vertical lines supported, but the OS doesn't need to draw anything else
    if (y1 == y2) {
        // Horizontal
        for (int x = x1; x < x2; x++) {
            draw_pixel(x, y1, colour);
        }
    } else if (x1 == x2) {
        // Vertical
        for (int y = y1; y < y2; y++) {
            draw_pixel(x1, y, colour);
        }
    }
}

void ILI9341Sprite::draw_sprite(uint16_t x, uint16_t y, ILI9341Sprite *other) {
    for (uint16_t ix = 0; ix < other->width; ix++) {
        for (uint16_t iy = 0; iy < other->height; iy++) {
            // Not using draw_pixel because that would flip the endianness
            // Because we're drawing from another sprite, the endianness was already flipped
            if ((ix + x) < width && (iy + y) < height) {
                data[(iy + y) * width + (ix + x)] = other->data[iy * other->width + ix];
            }
        }
    }
}

void ILI9341Sprite::draw_bitmap(uint16_t sx, uint16_t sy, uint16_t *bitmap) {
  if (bitmap == nullptr) return;

  uint16_t width = bitmap[0];
  uint16_t height = bitmap[1];
  uint16_t transparency = bitmap[2];
  uint16_t run_length = bitmap[3];

  int index = 4;
  for (uint16_t x = 0; x < width; x++) {
    for (uint16_t y = 0; y < height; y++) {
      if (bitmap[index] == run_length) {
        uint16_t times = bitmap[index + 1];
        uint16_t colour = bitmap[index + 2];

        if (colour != transparency) {
          for (uint16_t i = 0; i < times; i++) {
            draw_pixel(sx + x, sy + y + i, colour);
          }
        }

        y += times - 1;
        index += 3;
      } else {
        if (bitmap[index] != transparency) {
          draw_pixel(sx + x, sy + y, bitmap[index]);
        }
        index++;
      }
    }
  }
}


void ILI9341Sprite::draw_char(char character) {
    // Special case - move down by the height of one character
    if (character == '\n') {
        cursor_x = 0;
        cursor_y += font['A'][1];
        return;
    }

    uint8_t *character_bitmap = font[character];
    if (character_bitmap == NULL) return;

    // Each character is 4bpp;, so we maintain a flip-flopping boolean of whether to read the upper
    // or lower byte
    bool lower_byte = false;
    size_t idx = 2;
    for (int x = 0; x < character_bitmap[0]; x++) {
        for (int y = 0; y < character_bitmap[1]; y++) {
            uint8_t alpha_nibble;
            if (lower_byte) {
                alpha_nibble = character_bitmap[idx] & 0xF;
                lower_byte = false;
                idx++;
            } else {
                alpha_nibble = (character_bitmap[idx] & 0xF0) >> 4;
                lower_byte = true;
            }

            if (alpha_nibble != 0) {
                // Interpolate between the existing pixel (background colour) and the text colour,
                // using the font's alpha for this pixel, to make the anti-aliasing look good!
                // This is effectively alpha compositing, but it's a really simple case of it, since
                // our background always has maximum alpha.

                uint16_t background_colour = get_pixel(cursor_x + x, cursor_y + y);
                int8_t background_r = (background_colour & 0b1111100000000000) >> 11;
                int8_t background_g = (background_colour & 0b0000011111100000) >> 5;
                int8_t background_b = (background_colour & 0b0000000000011111);

                int8_t font_r = (font_colour & 0b1111100000000000) >> 11;
                int8_t font_g = (font_colour & 0b0000011111100000) >> 5;
                int8_t font_b = (font_colour & 0b0000000000011111);

                // 4bpp = 16 steps
                // Multiply integers by 8 while we're working with them, so that we have room to
                // spare on the truncating division
                int16_t step_r = (background_r * 8 - font_r * 8) / 16;
                int16_t step_g = (background_g * 8 - font_g * 8) / 16;
                int16_t step_b = (background_b * 8 - font_b * 8) / 16;

                int8_t composited_r = (int8_t)(background_r - (step_r * alpha_nibble) / 8);
                int8_t composited_g = (int8_t)(background_g - (step_g * alpha_nibble) / 8);
                int8_t composited_b = (int8_t)(background_b - (step_b * alpha_nibble) / 8);

                uint16_t colour = ((uint16_t)composited_r << 11) | ((uint16_t)composited_g << 5) | ((uint16_t)composited_b);
                draw_pixel(cursor_x + x, cursor_y + y, colour);
            }
        }
    }

    cursor_x += character_bitmap[0] - 1;
}

void ILI9341Sprite::draw_string(char *str) {
    size_t idx = 0;
    while (str[idx]) {
        draw_char(str[idx]);
        idx++;
    }
}

void ILI9341::begin() {
    // Turn on display
    gpio_init(power);
    gpio_set_dir(power, GPIO_OUT);
    gpio_put(power, 1);
    sleep_ms(100);

    // Chip-select display
    gpio_init(cs);
    gpio_set_dir(cs, GPIO_OUT);
    gpio_put(cs, 0);

    // Set up SPI and pins
    spi_init(spi0, 70000 * 1000);
    gpio_set_function(miso, GPIO_FUNC_SPI);
    gpio_set_function(mosi, GPIO_FUNC_SPI);
    gpio_set_function(sclk, GPIO_FUNC_SPI);
    gpio_init(dc);
    gpio_set_dir(dc, GPIO_OUT);

    // Hardware reset
    gpio_init(rst);
    gpio_set_dir(rst, GPIO_OUT);
    gpio_put(rst, 0);
    sleep_ms(50);
    gpio_put(rst, 1);
    sleep_ms(50);

    // Init sequence
    write_command(0x0f);
    write_data(0x03); write_data(0x80); write_data(0x02);
    write_command(0xcf);
    write_data(0x00); write_data(0xc1); write_data(0x30);
    write_command(0xed);
    write_data(0x64); write_data(0x03); write_data(0x12); write_data(0x81);
    write_command(0xe8);
    write_data(0x85); write_data(0x00); write_data(0x78);
    write_command(0xcb);
    write_data(0x39); write_data(0x2c); write_data(0x00); write_data(0x34); write_data(0x02);
    write_command(0xf7);
    write_data(0x20);
    write_command(0xea);
    write_data(0x00); write_data(0x00);
    write_command(0xc0);
    write_data(0x23);
    write_command(0xc1);
    write_data(0x10);
    write_command(0xc5);
    write_data(0x3e); write_data(0x28);
    write_command(0xc7);
    write_data(0x86);
    
    write_command(0x36);
    write_data(0x48);

    write_command(0x3a);
    write_data(0x55);
    write_command(0xb1);
    write_data(0x00); write_data(0x18);
    write_command(0xb6);
    write_data(0x08); write_data(0x82); write_data(0x27);
    write_command(0xf2);
    write_data(0x00);
    write_command(0x26);
    write_data(0x01);
    
    write_command(0xe0);
    write_data(0xf); write_data(0x31); write_data(0x2b); write_data(0xc); write_data(0xe); write_data(0x8); write_data(0x4e); write_data(0xf1); write_data(0x37); write_data(0x7); write_data(0x10); write_data(0x3); write_data(0xe); write_data(0x9); write_data(0x0);

    write_command(0xe1);
    write_data(0x0); write_data(0xe); write_data(0x14); write_data(0x3); write_data(0x11); write_data(0x7); write_data(0x31); write_data(0xc1); write_data(0x48); write_data(0x8); write_data(0xf); write_data(0xc); write_data(0x31); write_data(0x36); write_data(0xf);

    write_command(0x11); // Unsleep
    sleep_ms(150);
    write_command(0x29); // Display on
    sleep_ms(150);
}

ILI9341Sprite* ILI9341::create_sprite(uint16_t width, uint16_t height) {
    auto sprite = new ILI9341Sprite(width, height);
    sprite->allocate();
    sprite->fill(0);
    return sprite;
}

void ILI9341::draw_sprite(uint16_t x, uint16_t y, ILI9341Sprite *sprite) {
    uint16_t x2 = x + sprite->width + 1;
    uint16_t y2 = y + sprite->height + 1;

    // CASET
    write_command(0x2A);
    write_data((x & 0xFF00) >> 8);
    write_data(x & 0x00FF);
    write_data((x2 & 0xFF00) >> 8);
    write_data(x2 & 0x00FF);

    // PASET
    write_command(0x2B); 
    write_data((y & 0xFF00) >> 8);
    write_data(y & 0x00FF);
    write_data((y2 & 0xFF00) >> 8);
    write_data(y2 & 0x00FF);

    // RAMRW
    write_command(0x2C);

    write_data_fast_begin();
    for (int i = 0; i < sprite->height; i++) {
        write_data_fast_multiple(((uint8_t*)sprite->data) + (i * sprite->width * 2), sprite->width * 2);
    }
}

void ILI9341::write_command(uint8_t c) {
    gpio_put(dc, 0);
    spi_write_blocking(spi0, &c, 1);
}

void ILI9341::write_data(uint8_t d) {
    gpio_put(dc, 1);
    spi_write_blocking(spi0, &d, 1);
}
