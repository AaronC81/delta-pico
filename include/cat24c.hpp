#pragma once

#include <stdint.h>
#include "hardware/i2c.h"

class CAT24C {
public:
    CAT24C(i2c_inst_t *_i2c, uint8_t _i2cAddress) : i2c(_i2c), i2cAddress(_i2cAddress) {}

    bool connected();
    bool busy();
    bool write(uint16_t address, uint16_t count, const uint8_t *buffer);
    bool read(uint16_t address, uint16_t count, uint8_t *buffer);

    const uint16_t PAGE_SIZE = 64;
    const uint16_t PAGE_WRITE_MS = 5;
protected:
    i2c_inst_t *i2c;
    uint8_t i2cAddress;
};
