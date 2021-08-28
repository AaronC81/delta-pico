#pragma once

#include <Wire.h>
#include "pcf8574.hpp"

extern "C" {
    #include "delta_pico_rust.h"
}

enum class ButtonEvent {
    PRESS,
    RELEASE,
};

class ButtonMatrix {
public:
    const int ROWS = 7;
    const int COLS = 7;
    const int DEBOUNCE_MS = 20;
    
    // The row/col wiring doesn't exactly correspond to PCF8574 pin numbers.
    // This array maps a PCF8574 bit to a row/col number.
    const uint8_t PIN_MAPPING[7] = { 0, 1, 2, 3, 6, 5, 4 };

    ButtonMatrix(PCF8574 _row, PCF8574 _col) : row(_row), col(_col) {}

    void begin(void);

    bool getRawButton(uint8_t &pressedRow, uint8_t &pressedCol);
    bool waitForEvent(uint8_t &pressedRow, uint8_t &pressedCol, ButtonEvent &event);

    bool waitForEventInput(RbopInput &input, ButtonEvent &event);

protected:
    PCF8574 row, col;

    bool currentlyPressed = false;
    uint8_t currentlyPressedRow = 0;
    uint8_t currentlyPressedCol = 0;
    unsigned long currentlyPressedTime = 0;
};
