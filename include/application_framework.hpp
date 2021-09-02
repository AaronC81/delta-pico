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
    uint16_t *_spriteData;
    CAT24C *_storage;
};
