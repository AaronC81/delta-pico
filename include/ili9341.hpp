#pragma once

#include <stdint.h>
#include "hardware/spi.h"

class ILI9341Sprite {
public:
    ILI9341Sprite(uint16_t _width, uint16_t _height) : width(_width), height(_height) {}

    void allocate();
    void free();

protected:
    uint16_t *data;
    uint16_t width, height;
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

protected:
    spi_inst_t *spi;
    uint8_t miso, mosi, sclk, dc, cs, rst, power;
};
