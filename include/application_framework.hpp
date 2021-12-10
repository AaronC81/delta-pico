#pragma once

#include "hardware.hpp"
#include "pcf8574.hpp"
#include "button_matrix.hpp"
#include "animate.hpp"
#include "cat24c.hpp"
#include "ili9341.hpp"

class ApplicationFramework {
public:
    void initialize();
    void draw();
    ILI9341Sprite* newSprite(int16_t width, int16_t height);
    void freeSprite(ILI9341Sprite* sprite);
    void switchToSprite(ILI9341Sprite *new_sprite);
    void switchToScreen();

    ButtonMatrix&  buttons() const;
    ILI9341Sprite& sprite()  const;
    CAT24C&        storage() const;
    ILI9341&       tft()     const;

    static ApplicationFramework instance;
    
private:
    ApplicationFramework() {}

    bool _initialized = false;

    ILI9341 *_tft;
    PCF8574 *_colPcf, *_rowPcf;
    ButtonMatrix *_buttons;
    ILI9341Sprite *_sprite;
    ILI9341Sprite *_screenSprite;
    CAT24C *_storage;
};
