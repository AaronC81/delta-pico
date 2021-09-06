#include "application_framework.hpp"

extern "C" {
    #include <DroidSans-20.h>
}

void ApplicationFramework::initialize() {
    if (_initialized) return;

    _tft = new TFT_eSPI();
    _i2c = new arduino::MbedI2C(I2C_SDA_PIN, I2C_SCL_PIN);
    _colPcf = new PCF8574(*_i2c, I2C_EXPANDER_ADDRESS_1);
    _rowPcf = new PCF8574(*_i2c, I2C_EXPANDER_ADDRESS_2);
    _buttons = new ButtonMatrix(*_rowPcf, *_colPcf);
    _screenSprite = newSprite(TFT_WIDTH, TFT_HEIGHT);
    _sprite = _screenSprite;
    _storage = new CAT24C(*_i2c, CAT24C_ADDRESS);

    _i2c->begin();
    _buttons->begin();

    _tft->init();
    _tft->fillScreen(TFT_BLACK);
    _tft->initDMA();
    _tft->setRotation(0);

    switchToScreen();

    _initialized = true;
}

void ApplicationFramework::draw() {
    _tft->startWrite();
    _tft->pushImageDMA(0, 0, IWIDTH, IHEIGHT, (uint16_t*)_sprite->getPointer());
    _tft->endWrite();
}

TFT_eSprite* ApplicationFramework::newSprite(int16_t width, int16_t height) {
    auto sprite = new TFT_eSprite(_tft);
    sprite->setColorDepth(COLOR_DEPTH);
    sprite->createSprite(width, height);

    sprite->loadFont(DroidSans_20_vlw);
    sprite->setTextColor(TFT_WHITE);
    sprite->setTextDatum(MC_DATUM);
    sprite->setTextWrap(false, false);

    return sprite;
}

void ApplicationFramework::freeSprite(TFT_eSprite *sprite) {
    sprite->deleteSprite();
}

void ApplicationFramework::switchToSprite(TFT_eSprite *sprite) {
    _sprite = sprite;
}

void ApplicationFramework::switchToScreen() {
    _sprite = _screenSprite;
}

ButtonMatrix& ApplicationFramework::buttons() const { return *_buttons; }
TFT_eSprite&  ApplicationFramework::sprite()  const { return *_sprite;  }
CAT24C&       ApplicationFramework::storage() const { return *_storage; }

ApplicationFramework ApplicationFramework::instance = {};
