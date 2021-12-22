cmake_minimum_required(VERSION 3.12)

# Hardware revision you're building for - pick one!
#add_definitions(-DDELTA_PICO_PROTOTYPE)
add_definitions(-DDELTA_PICO_REV1)

# Pull in PICO SDK (must be before project)
include(pico_sdk_import.cmake)

project(pico_examples C CXX ASM)
set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)

# Initialize the SDK
pico_sdk_init()

add_executable(delta-pico
  src/main.cpp
  src/hardware.cpp
  src/button_matrix.cpp
  src/cat24c.cpp
  src/ili9341.cpp
  src/pcf8574.cpp
)

include_directories(include res font rust/target/include)

# Make the stdio functions use USB rather than the serial pins
pico_enable_stdio_usb(delta-pico 1)
pico_enable_stdio_uart(delta-pico 0)

# Pull in our pico_stdlib which pulls in commonly used features
target_link_libraries(delta-pico pico_stdlib hardware_spi hardware_i2c hardware_adc pico_time)

# create map/bin/hex file etc.
pico_add_extra_outputs(delta-pico)

# Build the Rust component
add_custom_target(build_bridge ALL COMMAND python3 ../build_bridge.py)

# Link against Rust component
target_link_libraries(delta-pico
  ${CMAKE_CURRENT_SOURCE_DIR}/rust/target/thumbv6m-none-eabi/release/libdelta_pico_rust.a
)

# Build resources and fonts
add_custom_target(build_res ALL COMMAND python3 ../build_res.py)
add_custom_target(build_font ALL COMMAND python3 ../build_font.py)
