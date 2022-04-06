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

The Delta Pico's software is written in Rust. The hardware abstraction layer is
separate to the user software, theoretically allowing the operating system to
run on other hardware which supports embedded Rust!

![Ferris, the Rust mascot, a crab, sitting on top of the Delta
Pico](img/ferris.jpg)

## Building

### Dependencies

You will likely need a bleeding-edge checkout of rbop, so you should check this
out too and adjust the path in `rust/Cargo.toml` accordingly.

This project has a dependency on FontForge, which is used for compiling fonts
into bitmaps. Install FontForge, and then set the `DELTA_PICO_FFPYTHON`
environment variable to point to its included `ffpython` executable. If you've
done this right, you should be able to run `$DELTA_PICO_FFPYTHON` in your 
terminal (or `%DELTA_PICO_FFPYTHON%` on Windows) and get a Python interactive
prompt.

The build script also uses [elf2uf2-rs](https://github.com/JoNil/elf2uf2-rs) to
flash software to the Pico, so be sure to install this.

### Building and Flashing

With all of the above dependencies sorted, cd into `rust-hal` and run:

```
cargo run
```

With a Pico connected in bootloader mode, this should build the project and
flash it onto your Pico.

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

## License

This repository is licensed entirely under the [MIT License](LICENSE), except
for `font/DroidSans.ttf`, which is sourced from Adobe Fonts and licensed under
the [Apache License](https://fonts.adobe.com/variations/1291/eula).
