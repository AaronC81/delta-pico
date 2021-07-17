#pragma once

#include <Wire.h>
#include "pcf8574.hpp"

class ButtonMatrix {
public:
    const int ROWS = 7;
    const int COLS = 7;
    
    // The row/col wiring doesn't exactly correspond to PCF8574 pin numbers.
    // This array maps a PCF8574 bit to a row/col number.
    const uint8_t PIN_MAPPING[7] = { 0, 1, 2, 3, 6, 5, 4 };

    ButtonMatrix(PCF8574 _row, PCF8574 _col) : row(_row), col(_col) {}

    void begin(void);
    bool getButton(uint8_t &row, uint8_t &col);
protected:
    PCF8574 row, col;
};
