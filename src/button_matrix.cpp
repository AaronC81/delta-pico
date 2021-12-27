#include "button_matrix.hpp"
#include "hardware.h"

#include "pico/time.h"
#include "pico/multicore.h"

void ButtonMatrix::begin(void) {
    // Set to input
    col.write(0xFF);
}

bool ButtonMatrix::get_raw_button(uint8_t &pressed_row, uint8_t &pressed_col) {
    for (uint8_t r = 0; r < ROWS; r++) {
        // Set all bits except this row
        uint8_t row_value = (uint8_t)~(1 << r);
        row.write(row_value);

        // TODO: bad, but needed!
        sleep_ms(1);

        // Check if any buttons in this row were pressed
        uint8_t byte = col.read();
        byte = (uint8_t)(~byte);
        if (byte > 0) {
            // Yes! Log2 to find out which col it is
            pressed_col = 0;
            while (byte >>= 1) ++pressed_col;

            // Return the row too
            pressed_row = r;

            // Map row and column to actual numbers, rather than PCF8574 wiring
            pressed_col = PIN_MAPPING[pressed_col];
            pressed_row = PIN_MAPPING[pressed_row];

            // Indicate to the caller that a button was pressed
            return true;
        }
    }

    // Nothing pressed
    return false;
}

bool ButtonMatrix::get_event(uint8_t &event_row, uint8_t &event_col, ButtonEvent &event, bool wait) {
    // Was a button already being pressed?
    if (currently_pressed) {
        // Is it no longer pressed?
        if (!get_raw_button(event_row, event_col)) {
            // Is it still no longer pressed after the debounce time?
            sleep_ms(DEBOUNCE_MS);
            if (!get_raw_button(event_row, event_col))
            {
                // The button has been released!
                currently_pressed = false;
                event = ButtonEvent::Release;
                event_row = currently_pressed_row;
                event_col = currently_pressed_col;
                return true;
            }
        }

        // Are we now pressing a different button instead?
        if (event_row != currently_pressed_row || event_col != currently_pressed_col) {
            // Fire a release now, and let the next iteration catch the new press
            currently_pressed = false;
            event = ButtonEvent::Release;
            event_row = currently_pressed_row;
            event_col = currently_pressed_col;
            return true;
        }

        // Nothing happened
        return false;
    }

    if (wait) {
        // Wait for a button to be pressed
        while (!get_raw_button(event_row, event_col));
    } else {
        if (!get_raw_button(event_row, event_col)) {
            return false;
        }
    }

    // Is it still pressed after the debounce time?
    uint8_t now_event_row, now_event_col;
    sleep_ms(DEBOUNCE_MS);
    if (get_raw_button(now_event_row, now_event_col)
        && event_row == now_event_row
        && event_col == now_event_col)
    {
        // A new button is pressed!
        currently_pressed = true;
        event = ButtonEvent::Press;
        currently_pressed_row = event_row;
        currently_pressed_col = event_col;
        currently_pressed_time = to_ms_since_boot(get_absolute_time());
        return true;
    }

    // Nothing happened
    return false;
}

bool ButtonMatrix::get_event_input(ButtonInput &input, ButtonEvent &event, bool wait) {
    recursive_mutex_enter_blocking(&i2c_mutex);

    uint8_t r, c;
    if (ButtonMatrix::get_event(r, c, event, wait)) {
        input = button_mapping[r][c];
        recursive_mutex_exit(&i2c_mutex);
        return true;
    } else {
        recursive_mutex_exit(&i2c_mutex);
        return false;
    }
}
