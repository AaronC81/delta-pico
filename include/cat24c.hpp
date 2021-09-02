#pragma once

#include <Wire.h>

class CAT24C {
public:
    CAT24C(arduino::MbedI2C &_wire, uint8_t _i2cAddress)
        : wire(_wire), i2cAddress(_i2cAddress) {}

    bool connected();
    bool busy();
    bool write(uint16_t address, uint8_t count, const uint8_t *buffer);
    bool read(uint16_t address, uint8_t count, uint8_t *buffer);

    const uint16_t PAGE_SIZE = 64;
    const uint16_t PAGE_WRITE_MS = 5;
protected:
    arduino::MbedI2C &wire;
    uint8_t i2cAddress;
};
