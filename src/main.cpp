#include <TFT_eSPI.h>
#include <Wire.h>

#include "hardware.hpp"
#include "pcf8574.hpp"
#include "button_matrix.hpp"
#include "animate.hpp"

#define USE_DMA_TO_TFT
#define COLOR_DEPTH 16

#define IWIDTH  320
#define IHEIGHT 240

#define CUBE_SIZE 200

TFT_eSPI tft = TFT_eSPI();

arduino::MbedI2C i2c(I2C_SDA_PIN, I2C_SCL_PIN);

PCF8574 colPcf(i2c, I2C_EXPANDER_ADDRESS_1);
PCF8574 rowPcf(i2c, I2C_EXPANDER_ADDRESS_2);

ButtonMatrix buttons(rowPcf, colPcf);

#define GRID_ITEMS 6
#define GRID_ROWS 2
#define GRID_COLS 3

#define GRID_SPRITE_WIDTH (IWIDTH / GRID_COLS)
#define GRID_SPRITE_HEIGHT (IHEIGHT / GRID_ROWS)

#define GRID_UNSELECTED_PADDING 12
#define GRID_SELECTED_PADDING 2

// x1, y1, x2, y2 
int grid_animate_start[4] = {
  GRID_UNSELECTED_PADDING,
  GRID_UNSELECTED_PADDING,
  GRID_SPRITE_WIDTH - GRID_UNSELECTED_PADDING * 2,
  GRID_SPRITE_HEIGHT - GRID_UNSELECTED_PADDING * 2
};
int grid_animate_end[4] = {
  GRID_SELECTED_PADDING,
  GRID_SELECTED_PADDING,
  GRID_SPRITE_WIDTH - GRID_SELECTED_PADDING * 2,
  GRID_SPRITE_HEIGHT - GRID_SELECTED_PADDING * 2
};

#define GRID_ANIMATE_TIME_FRAME 20

TFT_eSprite menuItemSprites[GRID_ITEMS] = {
  TFT_eSprite(&tft),
  TFT_eSprite(&tft),
  TFT_eSprite(&tft),
  TFT_eSprite(&tft),
  TFT_eSprite(&tft),
  TFT_eSprite(&tft),
};
uint16_t* menuItemSpriteData[GRID_ITEMS];
uint8_t menuItemSelection = 0;
Animate<4>* menuItemSelectionAnimation;

void drawUnselected(uint8_t idx) {
  menuItemSprites[idx].fillScreen(0);
  menuItemSprites[idx].fillRect(
    GRID_UNSELECTED_PADDING,
    GRID_UNSELECTED_PADDING,
    GRID_SPRITE_WIDTH - GRID_UNSELECTED_PADDING * 2,
    GRID_SPRITE_HEIGHT - GRID_UNSELECTED_PADDING * 2,
    0xFFFFFFFF
  );
}

void resetSelectedAnimation() {
  // Restore previously selected item
  drawUnselected(menuItemSelection);

  // Set up animation
  menuItemSelectionAnimation = new Animate<4>(
    grid_animate_start,
    grid_animate_end,
    GRID_ANIMATE_TIME_FRAME,
    Easing::EASE_OUT
  );
}

void setup() {
  Serial.begin(115200);
  i2c.begin();
  buttons.begin();

  tft.init();
  tft.fillScreen(TFT_BLACK);
  tft.initDMA();
  tft.setRotation(3);

  // Set up sprite
  for (int i = 0; i < GRID_ITEMS; i++) {
    menuItemSprites[i].setColorDepth(COLOR_DEPTH);
    menuItemSpriteData[i] = (uint16_t*)menuItemSprites[i].createSprite(
      GRID_SPRITE_WIDTH, GRID_SPRITE_HEIGHT
    );
    menuItemSprites[i].setTextColor(TFT_WHITE);
    menuItemSprites[i].setTextDatum(MC_DATUM);

    drawUnselected(i);
  }

  resetSelectedAnimation();
}

int y = 10;

void loop() {
  // Grab exclusive use of the SPI bus
  tft.startWrite();

  for (int row = 0; row < GRID_ROWS; row++) {
    for (int col = 0; col < GRID_COLS; col++) {
      int idx = row * GRID_COLS + col;
      if (idx == menuItemSelection) {
        menuItemSelectionAnimation->tick();
        int *cv = menuItemSelectionAnimation->currentValue;

        menuItemSprites[idx].fillScreen(0);
        menuItemSprites[idx].fillRect(
          cv[0],
          cv[1],
          cv[0] + cv[2],
          cv[1] + cv[3],
          0xFFFFFFFF
        );
      }

      tft.pushImageDMA(
        col * GRID_SPRITE_WIDTH,
        row * GRID_SPRITE_HEIGHT,
        GRID_SPRITE_WIDTH,
        GRID_SPRITE_HEIGHT,
        menuItemSpriteData[idx]
      );
    }
  }
  
  uint8_t r, c;
  ButtonEvent evt;
  if (buttons.waitForEvent(r, c, evt) && evt == ButtonEvent::PRESS) {
    resetSelectedAnimation();
    menuItemSelection++;
    menuItemSelection %= 6;
  }

  // Release bus
  tft.endWrite();
}
