# Delta Pico

The Delta Pico is a **powerful scientific calculator built around the Raspberry
Pi Pico**.

![The Delta Pico calculator in the centre of a wooden surface. Some of the
components of the calculator, such as its display and the Pico, are also beside
it.](img/table.jpg)

A follow-on project from my [Delta
M0](https://github.com/AaronC81/delta-m0), this calculator is much larger
and significantly more capable, with a 2.8" 240x320 colour display and 30 keys.

The Delta Pico is powered by the [rbop](https://github.com/AaronC81/rbop)
mathematical engine, for gorgeous textbook-style input.

![A hand holding the Delta Pico calculator, while it shows a calculation on the
display.](img/holding.jpg)

Most of the Delta Pico's software is written in Rust, with a small layer of C++
to glue it together with Arduino libraries.

![Ferris, the Rust mascot, a crab, sitting on top of the Delta
Pico](img/ferris.jpg)

## Building

This repository is a platform.io project, with a separate Cargo project in
the `rust` directory. The Cargo project is compiled automatically when using
`pio run`, so to build and flash this to your Pico:

```
pio run
picotool load .pio/build/pico/firmware.bin
picotool reboot
```

You will likely need a bleeding-edge checkout of rbop, so you should check this
out too and adjust the path in `rust/Cargo.toml` accordingly.

## Hardware

The KiCad files for the Delta Pico hardware can be found in the `cad` directory.

There is one error with the first hardware revision; a missing trace in the
button matrix, between the down arrow button and diode D8. This can be easily
fixed with a small piece of wire:

![A small piece of wire between the down arrow button and diode
D8.](img/wire-fix.png)

## Changelog
### Revision 1
- Initial revision!

### Revision 2
- Add I2C EEPROM chip
- Fix missing trace from "down" button to diode
- Connect display SD pins
- Move Pico further to the side to accommodate SD pins
- Rotate and move expansion socket to not be behind the display

### Revision 3
- Migrate to KiCad 6
- Button layout tweaks
    - The `(` key has been removed and merged into the `)` key to form a single `( )` key
        - rbop treats brackets as a block element, so it isn't possible to have a non-paired
          bracket
        - This means there is no reason to have a separate key for each bracket
    - In place of the `(` key, there is a new `TEXT` key
        - This will be used to toggle a multi-tap text entry mode on the numeric keypad
- Replace the expansion socket with a new debug port
    - 8 pins (4x2) rather than 4 pins
    - 2 of the extra pins are SWCLK and SWDIO (marked SWC and SWD respectively)
    - Other 2 are GPIO 18 and GPIO 19, currently unused
    - Moved to not be in the way of SD card slot
- Connect PCF8574 interrupt pins to GPIO 16/17
- Power-related tweaks
    - Broken out VSYS and GND near the JST battery connector
    - Add MOSFET to allow Pico to turn display on/off
        - Gate controllable with GPIO 22
    - Connect power switch to 3V3 and two GPIOs instead
        - The Pico is now always drawing battery power through VSYS
        - GPIO 26 or 27 will be high in the switch's on or off position respectively
        - In future, the device will soft-power-off by having the Pico turn off the display and
          enter DORMANT mode
    - Add Schottky diode on VSYS, to comply with the datasheet's suggestion
