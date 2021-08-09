#include <TFT_eSPI.h>
#include <Wire.h>

#include "hardware.hpp"
#include "pcf8574.hpp"
#include "button_matrix.hpp"
#include "animate.hpp"

extern "C" {
  #include <delta_pico_rust.h>
  #include <DroidSansMono-30.h>
}

#define USE_DMA_TO_TFT
#define COLOR_DEPTH 16

#define IWIDTH  320
#define IHEIGHT 240

#define SPAD 10
//#define SWIDTH (IWIDTH - SPAD * 2)
//#define SHEIGHT (IHEIGHT - SPAD * 2)
#define SWIDTH 100
#define SHEIGHT 100

TFT_eSPI tft = TFT_eSPI();

arduino::MbedI2C i2c(I2C_SDA_PIN, I2C_SCL_PIN);

PCF8574 colPcf(i2c, I2C_EXPANDER_ADDRESS_1);
PCF8574 rowPcf(i2c, I2C_EXPANDER_ADDRESS_2);

ButtonMatrix buttons(rowPcf, colPcf);

TFT_eSprite sprite(&tft);
uint16_t *spriteData;

#define I RbopInput

const RbopInput buttonMapping[7][7] = {
  { I::None,      I::MoveUp,    I::None,      I::None,      I::None,      I::None,      I::None, },
  { I::MoveLeft,  I::None,      I::MoveRight, I::None,      I::None,      I::None,      I::None, },
  { I::None,      I::MoveDown,  I::None,      I::None,      I::None,      I::None,      I::None, },
  { I::Digit7,    I::Digit8,    I::Digit9,    I::Delete,    I::None,      I::None,      I::None, },
  { I::Digit4,    I::Digit5,    I::Digit6,    I::Multiply,  I::Fraction,  I::None,      I::None, },
  { I::Digit1,    I::Digit2,    I::Digit3,    I::Add,       I::Subtract,  I::None,      I::None, },
  { I::Digit0,    I::Point,     I::None,      I::None,      I::None,      I::None,      I::None, },
};

#undef I

void rbopRendererClear() {
  sprite.fillScreen(TFT_BLACK);
}

void rbopRendererDrawLine(int64_t x1, int64_t y1, int64_t x2, int64_t y2) {
  sprite.drawLine(x1, y1, x2, y2, TFT_WHITE);
}

void rbopRendererDrawChar(int64_t x, int64_t y, uint8_t c) {
  sprite.setCursor(x, y);
  sprite.print((char)c);
}

RbopRendererInterface renderer = {
  .clear = rbopRendererClear,
  .draw_char = rbopRendererDrawChar,
  .draw_line = rbopRendererDrawLine,
};
RbopContext *ctx;

void rbopPanicHandler(const uint8_t *message) {
  sprite.setCursor(0, 0);
  sprite.println("PANIC!");
  sprite.println((const char*)message);
  tft.startWrite();
  tft.pushImageDMA(SPAD, SPAD, SWIDTH, SHEIGHT, spriteData);
  tft.endWrite();
}

void rbopDebugHandler(const uint8_t *message) {
  Serial.println((const char*)message);
}

void setup() {
  rbop_set_panic_handler(rbopPanicHandler);
  ctx = rbop_new(&renderer);
  rbop_set_viewport(ctx, SWIDTH, SHEIGHT);

  Serial.begin(115200);
  rbop_set_debug_handler(rbopDebugHandler);

  i2c.begin();
  buttons.begin();

  tft.init();
  tft.fillScreen(TFT_BLACK);
  tft.initDMA();
  tft.setRotation(3);

  sprite.setColorDepth(COLOR_DEPTH);
  spriteData = (uint16_t*)sprite.createSprite(SWIDTH, SHEIGHT);
  sprite.setTextColor(TFT_WHITE);
  sprite.setTextDatum(MC_DATUM);
  sprite.loadFont(DroidSansMono_30_vlw);
  sprite.setTextWrap(false, false);
}

void loop() {
  rbop_render(ctx);

  double result;
  if (rbop_evaluate(ctx, &result)) {
    sprite.setCursor(0, SHEIGHT - 30);
    sprite.print(result);
  }

  tft.startWrite();
  tft.pushImageDMA(SPAD, SPAD, SWIDTH, SHEIGHT, spriteData);
  tft.endWrite();

  uint8_t r, c;
  ButtonEvent evt;
  if (buttons.waitForEvent(r, c, evt) && evt == ButtonEvent::PRESS) {
    RbopInput input;

    rbop_input(ctx, buttonMapping[r][c]);
  }
}
