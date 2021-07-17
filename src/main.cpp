#include <TFT_eSPI.h>
#include <Wire.h>

#include "hardware.h"
#include "pcf8574.hpp"
#include "button_matrix.h"
#include "animate.hpp"

#define USE_DMA_TO_TFT
#define COLOR_DEPTH 16

#define IWIDTH  240
#define IHEIGHT 320

#define CUBE_SIZE 200

TFT_eSPI tft = TFT_eSPI();

TFT_eSprite sprite = TFT_eSprite(&tft);
uint16_t* spriteData;

arduino::MbedI2C i2c(I2C_SDA_PIN, I2C_SCL_PIN);

PCF8574 colPcf(i2c, I2C_EXPANDER_ADDRESS_1);
PCF8574 rowPcf(i2c, I2C_EXPANDER_ADDRESS_2);

ButtonMatrix buttons(rowPcf, colPcf);

void setup() {
  Serial.begin(115200);
  i2c.begin();
  buttons.begin();

  tft.init();
  tft.fillScreen(TFT_BLACK);
  tft.initDMA();

  // Set up sprite
  sprite.setColorDepth(COLOR_DEPTH);
  spriteData = (uint16_t*)sprite.createSprite(IWIDTH, IHEIGHT);
  sprite.setTextColor(TFT_BLACK);
  sprite.setTextDatum(MC_DATUM);

  sleep_ms(3000);
  Serial.println("Go!");
}

void loop() {
  // Grab exclusive use of the SPI bus
  tft.startWrite();

  // Draw something
  /*
  sprite.fillScreen(0);
  for (int i = 0; i < 128; i++) {
    sprite.fillRect(random(IWIDTH), random(IHEIGHT), 20, 20, random(INT16_MAX));
  }
  tft.pushImageDMA(0, 0, IWIDTH, IHEIGHT, spriteData);
  */

  //                     X1, X2,  Y1,  Y2
  int startPoints[4] = { 40, 60,  10,  80 };
  int endPoints[4]   = { 10, 120, 100, 200 };

  Animate<4> ease(startPoints, endPoints, 20, Easing::EASE_OUT);

  while (ease.tick()) {
    sprite.fillScreen(0);
    sprite.fillRect(
      ease.currentValue[0],
      ease.currentValue[2],
      ease.currentValue[1] - ease.currentValue[0],
      ease.currentValue[3] - ease.currentValue[2],
      0xFFFFFFFF
    );
    tft.pushImageDMA(0, 0, IWIDTH, IHEIGHT, spriteData);
    sleep_ms(10);
  }
  
  // Release bus
  tft.endWrite();

  uint8_t r, c;
  if (buttons.getButton(r, c)) {
    Serial.println(r);
    Serial.println(c);
    Serial.println("-----");
  }
}
