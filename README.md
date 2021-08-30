# Delta Pico

The Delta Pico is a **powerful scientific calculator built around the Raspberry
Pi Pico**.

![The Delta Pico calculator in the centre of a wooden surface. Some of the
components of the calculator, such as its display and the Pico, are also beside
it.](img/table.jpg)

A follow-on project from my [Delta
Zero](https://github.com/AaronC81/delta-zero), this calculator is much larger
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
