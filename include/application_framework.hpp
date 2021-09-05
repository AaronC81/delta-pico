#pragma once

#include <TFT_eSPI.h>
#include <Wire.h>

#include "hardware.hpp"
#include "pcf8574.hpp"
#include "button_matrix.hpp"
#include "animate.hpp"
#include "cat24c.hpp"

class ApplicationFramework {
public:
    void initialize();
    void draw();
    TFT_eSprite* newSprite(int16_t width, int16_t height);
    void freeSprite(TFT_eSprite* sprite);
    void switchToSprite(TFT_eSprite *new_sprite);
    void switchToScreen();

    ButtonMatrix& buttons() const;
    TFT_eSprite&  sprite()  const;
    CAT24C&       storage() const;

    static ApplicationFramework instance;
    
private:
    ApplicationFramework() {}

    bool _initialized = false;

    TFT_eSPI *_tft;
    arduino::MbedI2C *_i2c;
    PCF8574 *_colPcf, *_rowPcf;
    ButtonMatrix *_buttons;
    TFT_eSprite *_sprite;
    TFT_eSprite *_screenSprite;
    CAT24C *_storage;
};
