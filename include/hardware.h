#pragma once

#include "delta_pico_rust.h"
#include "pico/multicore.h"

#define I2C_SDA_PIN 20
#define I2C_SCL_PIN 21

#define I2C_EXPANDER_ADDRESS_1 0x38
#define I2C_EXPANDER_ADDRESS_2 0x3E

#define ILI9341_MISO_PIN 0
#define ILI9341_MOSI_PIN 3
#define ILI9341_SCLK_PIN 2
#define ILI9341_DC_PIN 5
#define ILI9341_CS_PIN 4
#define ILI9341_RST_PIN 6
#define ILI9341_POWER_PIN 28

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

// Yep, these are ours :)
// https://pid.codes/1209/DE1A/
#define USB_VID 0x1209
#define USB_PID 0xDE1A

// Same as Pico SDK's stdio
#define USB_INTERRUPT_IRQ 31
#define USB_INTERRUPT_INTERVAL_US 1000

#if defined(DELTA_PICO_PROTOTYPE)
#define DELTA_PICO_KEYPAD_PROTOTYPE
#define DELTA_PICO_REVISION_NAME "Prototype"

#elif defined(DELTA_PICO_REV1)
#define DELTA_PICO_KEYPAD_5x6
#define DELTA_PICO_REVISION_NAME "Rev. 1"

#elif defined(DELTA_PICO_REV2)
#define DELTA_PICO_KEYPAD_5x6
#define DELTA_PICO_REVISION_NAME "Rev. 2"

#elif defined(DELTA_PICO_REV3)
#define DELTA_PICO_KEYPAD_5x6
#define DELTA_PICO_REVISION_NAME "Rev. 3"

// Revision 3 has a Schottky diode on VSYS
// Specifically this one, B140HW: https://www.diodes.com/assets/Datasheets/ds30670.pdf
// The forward voltage (and thus VSYS voltage drop from the battery) varies by current draw and
// temperature, but is nominally about 0.35V
// So, to get a correct battery reading, we'll need to add this voltage back on to whatever we read
// from the VSYS ADC
// (Value is in millivolts)
#define DELTA_PICO_TRAIT_BATTERY_VOLTAGE_DROP 350

#else
#error "Please define a revision"
#endif

extern const ButtonInput button_mapping[7][7];
extern recursive_mutex_t i2c_mutex;
