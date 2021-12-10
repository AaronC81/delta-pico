#include "pcf8574.hpp"

void PCF8574::write(uint8_t byte) {
    i2c_write_blocking(i2c, i2cAddress, &byte, 1, false);
}

uint8_t PCF8574::read(void) {
    uint8_t byte;
    i2c_read_blocking(i2c, i2cAddress, &byte, 1, false);
    return byte;
}
