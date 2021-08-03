#include <TFT_eSPI.h>
#include <Wire.h>

#include "hardware.hpp"
#include "pcf8574.hpp"
#include "button_matrix.hpp"
#include "animate.hpp"

extern "C" {
  #include <rbop_bridge.h>
}

#define USE_DMA_TO_TFT
#define COLOR_DEPTH 16

#define IWIDTH  320
#define IHEIGHT 240

TFT_eSPI tft = TFT_eSPI();

arduino::MbedI2C i2c(I2C_SDA_PIN, I2C_SCL_PIN);

PCF8574 colPcf(i2c, I2C_EXPANDER_ADDRESS_1);
PCF8574 rowPcf(i2c, I2C_EXPANDER_ADDRESS_2);

ButtonMatrix buttons(rowPcf, colPcf);

TFT_eSprite sprite(&tft);
uint16_t *spriteData;

void rbopRendererClear() {
  sprite.fillScreen(TFT_BLACK);
}

void rbopRendererDrawLine(uint64_t x1, uint64_t y1, uint64_t x2, uint64_t y2) {
  sprite.drawLine(x1, y1, x2, y2, TFT_WHITE);
}

void rbopRendererDrawChar(uint64_t x, uint64_t y, uint8_t c) {
  sprite.setCursor(x, y);
  sprite.print((char)c);
}

void rbopPanicHandler(const uint8_t *message) {
  sprite.setCursor(0, 0);
  sprite.println("PANIC!");
  sprite.println((const char*)message);
  tft.startWrite();
  tft.pushImageDMA(10, 10, 100, 100, spriteData);
  tft.endWrite();
}

void setup() {
  rbop_set_panic_handler(rbopPanicHandler);

  RbopContext *ctx = rbop_new();

  Serial.begin(115200);
  i2c.begin();
  buttons.begin();

  tft.init();
  tft.fillScreen(TFT_BLACK);
  tft.initDMA();
  tft.setRotation(3);

  sprite.setColorDepth(COLOR_DEPTH);
  spriteData = (uint16_t*)sprite.createSprite(100, 100);
  sprite.setTextColor(TFT_WHITE);
  sprite.setTextDatum(MC_DATUM);

  RbopRendererInterface renderer = {
    .clear = rbopRendererClear,
    .draw_char = rbopRendererDrawChar,
    .draw_line = rbopRendererDrawLine,
  };

  rbop_foo(ctx);
  rbop_render(ctx, &renderer);

  tft.startWrite();
  tft.pushImageDMA(10, 10, 100, 100, spriteData);
  tft.endWrite();
}

void loop() {}
