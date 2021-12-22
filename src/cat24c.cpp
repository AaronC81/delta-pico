#include "cat24c.hpp"

#include <string.h>

bool CAT24C::connected() {
    uint8_t b;
    return i2c_read_blocking(i2c, i2cAddress, &b, 1, false) != PICO_ERROR_GENERIC;
}

bool CAT24C::busy() {
    // When busy, the device essentially falls off the bus
    return !connected();
}

bool CAT24C::read(uint16_t address, uint8_t count, uint8_t *buffer) {
    uint8_t bytes[] = { address >> 8, address & 0xFF };
    if (i2c_write_blocking(i2c, i2cAddress, bytes, 2, false) == PICO_ERROR_GENERIC) return false;

    i2c_read_blocking(i2c, i2cAddress, buffer, count, false);

    return true;
}

bool CAT24C::write(uint16_t address, uint8_t count, const uint8_t *buffer) {
    // Adapted from Qwiic EEPROM Arduino library

    //Break the buffer into page sized chunks
    uint16_t recorded = 0;
    while (recorded < count)
    {
        //Limit the amount to write to either the page size or the Arduino limit of 30
        int amt_to_write = count - recorded;

        if (amt_to_write > 1)
        {
            //Check for crossing of a page line. Writes cannot cross a page line.
            uint16_t page_number_1 = (address + recorded) / PAGE_SIZE;
            uint16_t page_number_2 = (address + recorded + amt_to_write - 1) / PAGE_SIZE;
            if (page_number_2 > page_number_1)
                amt_to_write = (page_number_2 * PAGE_SIZE) - (address + recorded); //Limit the read amt to go right up to edge of page barrier
        }

        //See if EEPROM is available or still writing a previous request
        while (busy()) //Poll device
            sleep_us(100);          //This shortens the amount of time waiting between writes but hammers the I2C bus

        uint8_t write_buffer[count + 2];
        write_buffer[0] = (uint8_t)((address + recorded) >> 8);
        write_buffer[1] = (uint8_t)((address + recorded) & 0xFF);
        memcpy(write_buffer + 2, buffer, count);

        if (i2c_write_blocking(i2c, i2cAddress, write_buffer, count + 2, false) != count + 2) return false;

        recorded += amt_to_write;

        sleep_ms(PAGE_WRITE_MS); //Delay the amount of time to record a page
    }

    return true;
}
