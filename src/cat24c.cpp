#include "cat24c.hpp"

bool CAT24C::connected() {
    wire.beginTransmission(i2cAddress);
    return !wire.endTransmission(i2cAddress);
}

bool CAT24C::busy() {
    return !connected();
}

bool CAT24C::read(uint16_t address, uint8_t count, uint8_t *buffer) {
    wire.beginTransmission(i2cAddress);
    wire.write(address >> 8);
    wire.write(address & 0xFF);
    if (wire.endTransmission()) return false;

    wire.requestFrom(i2cAddress, count);
    for (uint8_t i = 0; i < count; i++) {
        buffer[i] = wire.read();
    }

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
            delayMicroseconds(100);          //This shortens the amount of time waiting between writes but hammers the I2C bus

        wire.beginTransmission(i2cAddress);
        wire.write((uint8_t)((address + recorded) >> 8));   // MSB
        wire.write((uint8_t)((address + recorded) & 0xFF)); // LSB
        for (uint8_t x = 0; x < amtToWrite; x++)
            wire.write(buffer[recorded + x]);
        if (wire.endTransmission()) return false; //Send stop condition

        recorded += amtToWrite;

        delay(PAGE_WRITE_MS); //Delay the amount of time to record a page
    }

    return true;
}
