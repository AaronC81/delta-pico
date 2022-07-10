# HAL: Hardware Abstraction Layer (aka. Application Framework)

> Note: This was originally called the application framework, and is still named
> `ApplicationFramework` in code, as it was previously used to directly implement applications. Now
> the architecture is different, as the operating system sits in-between hardware and applications,
> but within the code the name has stuck around due to the refactoring effort needed to change it.

The HAL can be found as the `delta-pico-hal` crate, in the `rust-hal` directory. It has three main
responsibilities:

1. Implement `ApplicationFramework` to abstract away the details of the hardware
2. Implement a `main` which starts the OS (by calling `delta_pico_main`)
3. Provide a `global_allocator` so that the OS can allocate on the heap

The main interfaces provided by an `ApplicationFramework` implementation are for the display,
buttons, and storage. Having this common interface means that the HAL can be swapped out while
keeping the OS unchanged, allowing the OS to be completely hardware-agnostic!

The HAL is the binary crate, while the OS is a library crate. This is so that the HAL provides
`main`, which is important in embedded environments where there will be plenty of setup required
before the OS can start (e.g. creating a heap allocator, and peripheral setup).

## Delta Pico HAL

The HAL included in this repository is implemented for the physical Delta Pico. This HAL includes
low-level drivers for communicating with the CAT24C flash memory (over I2C), the PCF8574 logic
expanders used to drive the button matrix (also over I2C), and the ILI9341 display (over SPI).

This HAL utilises the dual-core configuration of the RP2040. Core 0 runs the OS, and drives the
display and storage whenever necessary. Core 1 is entirely dedicated to polling the button matrix
for new button presses; when a press is detected, it is placed into the inter-core FIFO queue. Core
0 will blockingly grab items from this queue when the OS needs user input.

This multi-core architecture is not strictly necessary, but it provides an improved user experience
by meaning that buttons pressed while the OS is busy are stored in a queue, rather than just
dropped.

# OS: Operating System

This is the user-facing software stack, which includes all of the applications, and a set of
higher-level drivers which they are implemented with.

As an example of how the level of abstraction between the HAL and OS drivers differs:

- The button driver provided by the HAL (`ButtonsInterface`) just reports press and release events.
- The `input` function provided by the OS intercepts button presses for system-level functions, such
  as `TEXT` and `MENU`, and deals with them transparently, without the application's involvement.
  It also provides an abstraction of "shifted button presses", whereas the HAL driver will just
  report a press of `SHIFT` and then a press of a regular button separately later.

"Operating system" is a rather generous name for this component - once an application is launched,
it doesn't really do anything besides provide a set of libraries for it to use. There's no
RTOS-style support for tasks or multitasking.

## The Borrow Checker

The relationship between the OS and applications is two-way; the OS launches applications, and
applications call back to the OS to use its APIs. Unfortunately, this isn't something the borrow
checker is able to model particularly well.

Instead, the OS uses `unsafe` code to pass around `OperatingSystemPointer` instances, which are
just raw pointers wrapped in a struct which implements `Deref` and `DerefMut`. This allows
shared, unchecked, mutable access to the OS from anywhere! That's a complete violation of the 
borrow checker's rules, but unless you do something unusual (like replace the entire OS with a new
instance), it shouldn't be too much of a problem.

One potential pitfall is an application making a call which replaces the current application, such
as `launch_application_by_name`. This can invalidate `self` while code is running within the
application, which is pretty much a guaranteed path to some undefined behaviour.
