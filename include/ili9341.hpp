#pragma once

#include <stdint.h>
#include "hardware/spi.h"
#include "hardware/gpio.h"

#include "hardware.h"

class ILI9341Sprite {
public:
    ILI9341Sprite(uint16_t _width, uint16_t _height)
        : width(_width), height(_height), cursor_x(0), cursor_y(0), font_colour(0) {}

    void allocate();
    void free();

    void fill(uint16_t colour);
    void draw_rect(uint16_t x, uint16_t y, uint16_t w, uint16_t h, uint16_t radius, bool filled, uint16_t colour);
    void draw_line(uint16_t x1, uint16_t y1, uint16_t x2, uint16_t y2, uint16_t colour);
    void draw_sprite(uint16_t x, uint16_t y, ILI9341Sprite *other);
    void draw_bitmap(uint16_t x, uint16_t y, uint16_t *bitmap);

    void draw_char(char character);
    void draw_string(char *str);

    // TODO: for the screen sprite, checking `x < width && y < height` rather than
    // `x < TFT_WIDTH && y < TFT_HEIGHT` costs us about 15ms of frame time!!
    // Can we special-case/optimise this somehow for the screen sprite?

    inline void draw_pixel(uint16_t x, uint16_t y, uint16_t colour) {
        if (x < width && y < height) {
            // Draw pixels with endianness flipped, since we assume this is the case when sending data
            // to the screen later
            data[y * width + x] = ((colour & 0xFF) << 8) | ((colour & 0xFF00) >> 8);
        }
    }

    inline uint16_t get_pixel(uint16_t x, uint16_t y) {
        if (x < width && y < height) {
            // Correct endianness after draw_pixel flips it
            uint16_t colour = data[y * width + x];
            return ((colour & 0xFF) << 8) | ((colour & 0xFF00) >> 8);
        } else {
            return 0;
        }
    }

    uint16_t width, height, cursor_x, cursor_y, font_colour;
    uint8_t **font;
    uint16_t *data;
};

class ILI9341 {
public:
    ILI9341(spi_inst_t *_spi, uint8_t _miso, uint8_t _mosi, uint8_t _sclk, uint8_t _dc, uint8_t _cs,
        uint8_t _rst, uint8_t _power) 
        : spi(_spi), miso(_miso), mosi(_mosi), sclk(_sclk), dc(_dc), cs(_cs), rst(_rst),
          power(_power) {}

    void begin();
    ILI9341Sprite* create_sprite(uint16_t width, uint16_t height);
    void draw_sprite(uint16_t x, uint16_t y, ILI9341Sprite *sprite);

    void write_data(uint8_t d);
    void write_command(uint8_t c);

    inline void write_data_fast_begin() {
        gpio_put(dc, 1);
    }

    inline void write_data_fast(uint8_t d) {
        spi_write_blocking(spi0, &d, 1);
    }

    inline void write_data_fast_multiple(uint8_t *d, size_t len) {
        spi_write_blocking(spi0, d, len);
    }

protected:
    spi_inst_t *spi;
    uint8_t miso, mosi, sclk, dc, cs, rst, power;
};
