#include "application_framework.hpp"

void ApplicationFramework::initialize() {
    if (_initialized) return;

    _tft = new TFT_eSPI();
    _i2c = new arduino::MbedI2C(I2C_SDA_PIN, I2C_SCL_PIN);
    _colPcf = new PCF8574(*_i2c, I2C_EXPANDER_ADDRESS_1);
    _rowPcf = new PCF8574(*_i2c, I2C_EXPANDER_ADDRESS_2);
    _buttons = new ButtonMatrix(*_rowPcf, *_colPcf);
    _sprite = new TFT_eSprite(_tft);

    _i2c->begin();
    _buttons->begin();

    _tft->init();
    _tft->fillScreen(TFT_BLACK);
    _tft->initDMA();
    _tft->setRotation(0);

    _sprite->setColorDepth(COLOR_DEPTH);
    _spriteData = (uint16_t*)_sprite->createSprite(SWIDTH, SHEIGHT);
    _sprite->setTextColor(TFT_WHITE);
    _sprite->setTextDatum(MC_DATUM);
    _sprite->setTextWrap(false, false);

    _initialized = true;
}

void ApplicationFramework::draw() {
    _tft->startWrite();
    _tft->pushImageDMA(SPAD, SPAD, SWIDTH, SHEIGHT, _spriteData);
    _tft->endWrite();
}

ButtonMatrix& ApplicationFramework::buttons() const { return *_buttons; }
TFT_eSprite&  ApplicationFramework::sprite()  const { return *_sprite;  }

ApplicationFramework ApplicationFramework::instance = {};
