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
  src/usb_mass_storage.c
)

include_directories(include res font rust/target/include)

# Pull in our libraries
target_link_libraries(delta-pico
  # Common stuff
  pico_stdlib
  
  # Hardware interfaces
  hardware_spi hardware_i2c hardware_adc pico_time hardware_interp
  
  # USB
  tinyusb_device tinyusb_board
  
  # Multicore (pico_util provides queues)
  pico_multicore pico_util
)

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
