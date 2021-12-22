#pragma once

#include <stdint.h>
#include "hardware/spi.h"
#include "hardware/gpio.h"

#include "hardware.hpp"

class ILI9341Sprite {
public:
    ILI9341Sprite(uint16_t _width, uint16_t _height)
        : width(_width), height(_height), cursorX(0), cursorY(0), fontColour(0) {}

    void allocate();
    void free();

    void fill(uint16_t colour);
    void drawRect(uint16_t x, uint16_t y, uint16_t w, uint16_t h, uint16_t radius, bool filled, uint16_t colour);
    void drawSprite(uint16_t x, uint16_t y, ILI9341Sprite *other);
    void drawBitmap(uint16_t x, uint16_t y, uint16_t *bitmap);

    void drawChar(char character);
    void drawString(char *str);

    // TODO: for the screen sprite, checking `x < width && y < height` rather than
    // `x < TFT_WIDTH && y < TFT_HEIGHT` costs us about 15ms of frame time!!
    // Can we special-case/optimise this somehow for the screen sprite?

    inline void drawPixel(uint16_t x, uint16_t y, uint16_t colour) {
        if (x < width && y < height) {
            // Draw pixels with endianness flipped, since we assume this is the case when sending data
            // to the screen later
            data[y * width + x] = ((colour & 0xFF) << 8) | ((colour & 0xFF00) >> 8);
        }
    }

    inline uint16_t getPixel(uint16_t x, uint16_t y) {
        if (x < width && y < height) {
            // Correct endianness after drawPixel flips it
            uint16_t colour = data[y * width + x];
            return ((colour & 0xFF) << 8) | ((colour & 0xFF00) >> 8);
        } else {
            return 0;
        }
    }

    uint16_t width, height, cursorX, cursorY, fontColour;
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
    ILI9341Sprite* createSprite(uint16_t width, uint16_t height);
    void drawSprite(uint16_t x, uint16_t y, ILI9341Sprite *sprite);

    void writeData(uint8_t d);
    void writeCommand(uint8_t c);

    inline void writeDataFastBegin() {
        gpio_put(dc, 1);
    }

    inline void writeDataFast(uint8_t d) {
        spi_write_blocking(spi0, &d, 1);
    }

    inline void writeDataFastMultiple(uint8_t *d, size_t len) {
        spi_write_blocking(spi0, d, len);
    }

protected:
    spi_inst_t *spi;
    uint8_t miso, mosi, sclk, dc, cs, rst, power;
};
