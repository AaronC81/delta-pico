#include "pcf8574.hpp"

void PCF8574::write(uint8_t byte) {
    wire.beginTransmission(i2cAddress);
    wire.write(byte);
    wire.endTransmission();
}

uint8_t PCF8574::read(void) {
    wire.requestFrom(i2cAddress, 1);
    return wire.read();
}
