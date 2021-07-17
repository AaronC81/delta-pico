#include "button_matrix.h"

void ButtonMatrix::begin(void) {
    // Set to input
    col.write(0xFF);
}

bool ButtonMatrix::getButton(uint8_t &pressedRow, uint8_t &pressedCol) {
    for (uint8_t r = 0; r < ROWS; r++) {
        // Set all bits except this row
        uint8_t rowValue = (uint8_t)~(1 << r);
        row.write(rowValue);

        // TODO: bad, but needed!
        sleep_ms(1);

        // Check if any buttons in this row were pressed
        uint8_t byte = col.read();
        byte = (uint8_t)(~byte);
        if (byte > 0) {
            // Yes! Log2 to find out which col it is
            pressedCol = 0;
            while (byte >>= 1) ++pressedCol;

            // Return the row too
            pressedRow = r;

            // Map row and column to actual numbers, rather than PCF8574 wiring
            pressedCol = PIN_MAPPING[pressedCol];
            pressedRow = PIN_MAPPING[pressedRow];

            // Indicate to the caller that a button was pressed
            return true;
        }
    }

    // Nothing pressed
    return false;
}
