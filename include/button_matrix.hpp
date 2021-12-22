#pragma once

#include "pcf8574.hpp"

extern "C" {
    #include "delta_pico_rust.h"
}

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

    bool get_raw_button(uint8_t &pressed_row, uint8_t &pressed_col);
    bool get_event(uint8_t &pressed_row, uint8_t &pressed_col, ButtonEvent &event, bool wait);

    bool get_event_input(ButtonInput &input, ButtonEvent &event, bool wait);

protected:
    PCF8574 row, col;

    bool currently_pressed = false;
    uint8_t currently_pressed_row = 0;
    uint8_t currently_pressed_col = 0;
    unsigned long currently_pressed_time = 0;
};
