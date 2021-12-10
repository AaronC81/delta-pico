#pragma once

#include <stdint.h>
#include "hardware/i2c.h"

class PCF8574 {
public:
    PCF8574(i2c_inst_t *_i2c, uint8_t _i2cAddress) : i2c(_i2c), i2cAddress(_i2cAddress) {}

    void write(uint8_t byte);
    uint8_t read(void);
protected:
    i2c_inst_t *i2c;
    uint8_t i2cAddress;
};
