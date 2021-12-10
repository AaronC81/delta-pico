#pragma once

extern "C" {
    #include "delta_pico_rust.h"
}

#define I2C_SDA_PIN 20
#define I2C_SCL_PIN 21

#define I2C_EXPANDER_ADDRESS_1 0x38
#define I2C_EXPANDER_ADDRESS_2 0x3E

#define CAT24C_ADDRESS 0x50

#define COLOR_DEPTH 16
#define USE_8BPP

#ifdef USE_8BPP
#define SOFTWARE_COLOR_DEPTH 8
#else
#define SOFTWARE_COLOR_DEPTH 16
#endif

#define TFT_WIDTH  240
#define TFT_HEIGHT 320

extern const ButtonInput buttonMapping[7][7];
