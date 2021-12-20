#include "application_framework.hpp"
#include "hardware.hpp"
#include "ili9341.hpp"

#include "hardware/gpio.h"
#include "hardware/i2c.h"

extern "C" {
    #include <DroidSans-20.h>
}

void ApplicationFramework::initialize() {
    if (_initialized) return;

    // TODO: put in hardware.h
    _tft = new ILI9341(
        spi0,
        0, // MISO
        3, // MOSI
        2, // SCLK
        5, // DC
        4, // CS
        6, // RST
        28 // Power
    );

    // Initialize I2C bus
    gpio_set_function(I2C_SDA_PIN, GPIO_FUNC_I2C);
    gpio_set_function(I2C_SCL_PIN, GPIO_FUNC_I2C);
    gpio_pull_up(I2C_SDA_PIN);
    gpio_pull_up(I2C_SCL_PIN);
    i2c_init(i2c0, 1000000);

    _colPcf = new PCF8574(i2c0, I2C_EXPANDER_ADDRESS_1);
    _rowPcf = new PCF8574(i2c0, I2C_EXPANDER_ADDRESS_2);
    _buttons = new ButtonMatrix(*_rowPcf, *_colPcf);
    _screenSprite = _tft->createSprite(TFT_WIDTH, TFT_HEIGHT);
    _sprite = _screenSprite;
    _storage = new CAT24C(i2c0, CAT24C_ADDRESS);

    _buttons->begin();
    _tft->begin();

    // TODO
    // _tft->fillScreen(TFT_BLACK);
    // _tft->initDMA();
    // _tft->setRotation(0);

    switchToScreen();

    _initialized = true;
}

void ApplicationFramework::draw() {
    _tft->drawSprite(0, 0, _screenSprite);
}

ILI9341Sprite* ApplicationFramework::newSprite(int16_t width, int16_t height) {
    return _tft->createSprite(width, height);
}

void ApplicationFramework::freeSprite(ILI9341Sprite *sprite) {
    sprite->free();
    delete sprite;
}

void ApplicationFramework::switchToSprite(ILI9341Sprite *sprite) {
    _sprite = sprite;
}

void ApplicationFramework::switchToScreen() {
    _sprite = _screenSprite;
}

ButtonMatrix&  ApplicationFramework::buttons() const { return *_buttons; }
ILI9341Sprite& ApplicationFramework::sprite()  const { return *_sprite;  }
ILI9341&       ApplicationFramework::tft()     const { return *_tft;  }
CAT24C&        ApplicationFramework::storage() const { return *_storage; }

ApplicationFramework ApplicationFramework::instance = {};
