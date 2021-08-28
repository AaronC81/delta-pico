#include "button_matrix.hpp"
#include "hardware.hpp"

void ButtonMatrix::begin(void) {
    // Set to input
    col.write(0xFF);
}

bool ButtonMatrix::getRawButton(uint8_t &pressedRow, uint8_t &pressedCol) {
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

bool ButtonMatrix::waitForEvent(uint8_t &eventRow, uint8_t &eventCol, ButtonEvent &event) {
    // Was a button already being pressed?
    if (currentlyPressed) {
        // Is it no longer pressed?
        if (!getRawButton(eventRow, eventCol)) {
            // Is it still no longer pressed after the debounce time?
            sleep_ms(DEBOUNCE_MS);
            if (!getRawButton(eventRow, eventCol))
            {
                // The button has been released!
                currentlyPressed = false;
                event = ButtonEvent::RELEASE;
                eventRow = currentlyPressedRow;
                eventCol = currentlyPressedCol;
                return true;
            }
        }

        // Nothing happened
        return false;
    }

    // Wait for a button to be pressed
    while (!getRawButton(eventRow, eventCol));

    // Is it still pressed after the debounce time?
    uint8_t nowEventRow, nowEventCol;
    sleep_ms(DEBOUNCE_MS);
    if (getRawButton(nowEventRow, nowEventCol)
        && eventRow == nowEventRow
        && eventCol == nowEventCol)
    {
        // A new button is pressed!
        currentlyPressed = true;
        event = ButtonEvent::PRESS;
        currentlyPressedRow = eventRow;
        currentlyPressedCol = eventCol;
        currentlyPressedTime = millis();
        return true;
    }

    // Nothing happened
    return false;
}

bool ButtonMatrix::waitForEventInput(RbopInput &input, ButtonEvent &event) {
    uint8_t r, c;
    if (ButtonMatrix::waitForEvent(r, c, event)) {
        input = buttonMapping[r][c];
        return true;
    } else {
        return false;
    }
}
