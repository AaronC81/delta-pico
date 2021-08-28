#pragma once

extern "C" {
    #include "delta_pico_rust.h"
}

#define I2C_SDA_PIN 20
#define I2C_SCL_PIN 21

#define I2C_EXPANDER_ADDRESS_1 0x38
#define I2C_EXPANDER_ADDRESS_2 0x3E

#define USE_DMA_TO_TFT
#define COLOR_DEPTH 16

#define IWIDTH  240
#define IHEIGHT 320

#define SPAD 10
#define SWIDTH (IWIDTH - SPAD * 2)
#define SHEIGHT (IHEIGHT - SPAD * 2)

extern const RbopInput buttonMapping[7][7];
