#include "cat24c.hpp"

bool CAT24C::connected() {
    i2c_write_blocking(i2c, i2cAddress, 0, 0, false) != PICO_ERROR_GENERIC;
}

bool CAT24C::busy() {
    // When busy, the device essentially falls off the bus
    return !connected();
}

bool CAT24C::read(uint16_t address, uint8_t count, uint8_t *buffer) {
    uint8_t bytes[] = { address >> 8, address & 0xFF };
    if (i2c_write_blocking(i2c, i2cAddress, bytes, 2, false) != PICO_ERROR_GENERIC) return false;

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
        int amtToWrite = count - recorded;

        if (amtToWrite > 1)
        {
            //Check for crossing of a page line. Writes cannot cross a page line.
            uint16_t pageNumber1 = (address + recorded) / PAGE_SIZE;
            uint16_t pageNumber2 = (address + recorded + amtToWrite - 1) / PAGE_SIZE;
            if (pageNumber2 > pageNumber1)
                amtToWrite = (pageNumber2 * PAGE_SIZE) - (address + recorded); //Limit the read amt to go right up to edge of page barrier
        }

        //See if EEPROM is available or still writing a previous request
        while (busy()) //Poll device
            sleep_us(100);          //This shortens the amount of time waiting between writes but hammers the I2C bus

        uint8_t bytes[] = {
            (uint8_t)((address + recorded) >> 8),
            (uint8_t)((address + recorded) & 0xFF),
        };
        i2c_write_blocking(i2c, i2cAddress, bytes, 2, true);   // MSB
        for (uint8_t x = 0; x < amtToWrite; x++)
            i2c_write_blocking(i2c, i2cAddress, &buffer[recorded + x], 1, true);
        i2c_write_blocking(i2c, i2cAddress, NULL, 0, false);

        recorded += amtToWrite;

        sleep_ms(PAGE_WRITE_MS); //Delay the amount of time to record a page
    }

    return true;
}
