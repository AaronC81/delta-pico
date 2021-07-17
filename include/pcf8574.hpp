#pragma once

#include <Wire.h>

class PCF8574 {
public:
    PCF8574(arduino::MbedI2C &_wire, uint8_t _i2cAddress)
        : wire(_wire), i2cAddress(_i2cAddress) {}

    void write(uint8_t byte);
    uint8_t read(void);
protected:
    arduino::MbedI2C &wire;
    uint8_t i2cAddress;
};
