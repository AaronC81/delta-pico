#include "ili9341.hpp"

void ILI9341Sprite::allocate() {
    data = new uint16_t[width * height];
}

void ILI9341Sprite::free() {
    delete data;
}

void ILI9341Sprite::fill(uint16_t colour) {
    drawRect(0, 0, TFT_WIDTH, TFT_HEIGHT, 0, true, colour);
}

void ILI9341Sprite::drawRect(uint16_t x, uint16_t y, uint16_t w, uint16_t h, uint16_t radius, bool filled, uint16_t colour) {
    // TODO: radius is ignored
    // TODO: filled is ignored
    for (uint16_t ix = 0; ix < w; ix++) {
        for (uint16_t iy = 0; iy < h; iy++) {
            drawPixel(x + ix, y + iy, colour);
        }
    }
}

void ILI9341Sprite::drawChar(char character) {
    // TODO: Aliasing needs to be relative to underlying colour, not black
    // TODO: fontColour is ignored

    uint8_t *characterBitmap = font[character];
    if (characterBitmap == NULL) return;

    // Each character is 4bpp;, so we maintain a flip-flopping boolean of whether to read the upper
    // or lower byte
    bool lowerByte = false;
    size_t idx = 2;
    for (int x = 0; x < characterBitmap[0]; x++) {
        for (int y = 0; y < characterBitmap[1]; y++) {
            uint8_t colourNibble;
            if (lowerByte) {
                colourNibble = characterBitmap[idx] & 0xF;
                lowerByte = false;
                idx++;
            } else {
                colourNibble = (characterBitmap[idx] & 0xF0) >> 4;
                lowerByte = true;
            }

            if (colourNibble != 0) {
                uint16_t colour = (colourNibble << 12) | (colourNibble << 7) | (colourNibble << 2);
                drawPixel(cursorX + x, cursorY + y, colour);
            }
        }
    }

    cursorX += characterBitmap[0];
}

void ILI9341Sprite::drawString(char *str) {
    size_t idx = 0;
    while (str[idx]) {
        drawChar(str[idx]);
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
    writeCommand(0x0f);
    writeData(0x03); writeData(0x80); writeData(0x02);
    writeCommand(0xcf);
    writeData(0x00); writeData(0xc1); writeData(0x30);
    writeCommand(0xed);
    writeData(0x64); writeData(0x03); writeData(0x12); writeData(0x81);
    writeCommand(0xe8);
    writeData(0x85); writeData(0x00); writeData(0x78);
    writeCommand(0xcb);
    writeData(0x39); writeData(0x2c); writeData(0x00); writeData(0x34); writeData(0x02);
    writeCommand(0xf7);
    writeData(0x20);
    writeCommand(0xea);
    writeData(0x00); writeData(0x00);
    writeCommand(0xc0);
    writeData(0x23);
    writeCommand(0xc1);
    writeData(0x10);
    writeCommand(0xc5);
    writeData(0x3e); writeData(0x28);
    writeCommand(0xc7);
    writeData(0x86);
    
    writeCommand(0x36);
    writeData(0x48);

    writeCommand(0x3a);
    writeData(0x55);
    writeCommand(0xb1);
    writeData(0x00); writeData(0x18);
    writeCommand(0xb6);
    writeData(0x08); writeData(0x82); writeData(0x27);
    writeCommand(0xf2);
    writeData(0x00);
    writeCommand(0x26);
    writeData(0x01);
    
    writeCommand(0xe0);
    writeData(0xf); writeData(0x31); writeData(0x2b); writeData(0xc); writeData(0xe); writeData(0x8); writeData(0x4e); writeData(0xf1); writeData(0x37); writeData(0x7); writeData(0x10); writeData(0x3); writeData(0xe); writeData(0x9); writeData(0x0);

    writeCommand(0xe1);
    writeData(0x0); writeData(0xe); writeData(0x14); writeData(0x3); writeData(0x11); writeData(0x7); writeData(0x31); writeData(0xc1); writeData(0x48); writeData(0x8); writeData(0xf); writeData(0xc); writeData(0x31); writeData(0x36); writeData(0xf);

    writeCommand(0x11); // Unsleep
    sleep_ms(150);
    writeCommand(0x29); // Display on
    sleep_ms(150);
}

ILI9341Sprite* ILI9341::createSprite(uint16_t width, uint16_t height) {
    auto sprite = new ILI9341Sprite(width, height);
    sprite->allocate();
    return sprite;
}

void ILI9341::drawSprite(uint16_t x, uint16_t y, ILI9341Sprite *sprite) {
    uint16_t x2 = x + sprite->width + 1;
    uint16_t y2 = y + sprite->height + 1;

    // CASET
    writeCommand(0x2A);
    writeData((x & 0xFF00) >> 8);
    writeData(x & 0x00FF);
    writeData((x2 & 0xFF00) >> 8);
    writeData(x2 & 0x00FF);

    // PASET
    writeCommand(0x2B); 
    writeData((y & 0xFF00) >> 8);
    writeData(y & 0x00FF);
    writeData((y2 & 0xFF00) >> 8);
    writeData(y2 & 0x00FF);

    // RAMRW
    writeCommand(0x2C);

    writeDataFastBegin();
    for (int i = 0; i < sprite->height; i++) {
        writeDataFastMultiple(((uint8_t*)sprite->data) + (i * sprite->width * 2), sprite->width * 2);
    }
}

void ILI9341::writeCommand(uint8_t c) {
    gpio_put(dc, 0);
    spi_write_blocking(spi0, &c, 1);
}

void ILI9341::writeData(uint8_t d) {
    gpio_put(dc, 1);
    spi_write_blocking(spi0, &d, 1);
}
