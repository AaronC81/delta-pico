#include "bitmap.h"

#include "pico/time.h"
#include "pico/stdlib.h"
#include "hardware/gpio.h"
#include "hardware/adc.h"

#include "hardware.hpp"
#include "pcf8574.hpp"
#include "button_matrix.hpp"
#include "cat24c.hpp"
#include "ili9341.hpp"

#include <stdio.h>

extern "C" {
  #include <delta_pico_rust.h>
  #include <droid_sans_20.h>
}

ILI9341 tft(
  spi0,
  ILI9341_MISO_PIN,
  ILI9341_MOSI_PIN,
  ILI9341_SCLK_PIN,
  ILI9341_DC_PIN,
  ILI9341_CS_PIN,
  ILI9341_RST_PIN,
  ILI9341_POWER_PIN
);
ILI9341Sprite *sprite;
ILI9341Sprite *screen_sprite;

PCF8574 col_pcf(i2c0, I2C_EXPANDER_ADDRESS_1);
PCF8574 row_pcf(i2c0, I2C_EXPANDER_ADDRESS_2);
ButtonMatrix buttons(row_pcf, col_pcf);
CAT24C storage(i2c0, CAT24C_ADDRESS);

auto framework_interface = ApplicationFrameworkInterface {
  .debug_handler = [](const uint8_t *string) {
    printf("%s\n", string);
  },

  .millis = []() -> uint32_t { return to_ms_since_boot(get_absolute_time()); },
  .micros = []() -> uint32_t { return to_us_since_boot(get_absolute_time()); },

  .charge_status = []() -> int32_t {
    // Read from Pico's VSYS ADC
    // Then divide by resolution, times by Pico logical voltage, times by 3
    // (Voltage is divided by 3 - see Pico Datasheet section 4.4) 
    adc_select_input(3);
    int adc_reading = adc_read();
    float voltage = ((float)adc_reading / 1024.0) * 3.3 * 3;

    // Source: https://phantompilots.com/threads/how-does-lipo-voltage-relate-to-percent.13597/
    if (voltage > 4.5) {  
      return -1; // Connected over USB
    } else if (voltage > 4.13) {
      return 100;
    } else if (voltage > 4.06) {
      return 90;
    } else if (voltage > 3.99) {
      return 80;
    } else if (voltage > 3.92) {
      return 70;
    } else if (voltage > 3.85) {
      return 60;
    } else if (voltage > 3.78) {
      return 50;
    } else if (voltage > 3.71) {
      return 40;
    } else if (voltage > 3.64) {
      return 30;
    } else if (voltage > 3.57) {
      return 20;
    } else if (voltage > 3.5) {
      return 10;
    } else {
      return 0;
    }
  },
  .heap_usage = [](uint64_t* used, uint64_t* available) {
    // TODO
    // mbed_stats_heap_t heap_stats;
    // mbed_stats_heap_get(&heap_stats);

    // *used = heap_stats.current_size;
    // *available = heap_stats.reserved_size;
    *used = 1000;
    *available = 2000;
  },

  .display = DisplayInterface {
    .width = TFT_WIDTH,
    .height = TFT_HEIGHT,

    .new_sprite = [](int16_t w, int16_t h) {
      auto new_sprite = tft.create_sprite(w, h);

      // Inherit font from screen sprite
      new_sprite->font = screen_sprite->font;
      new_sprite->font_colour = screen_sprite->font_colour;

      return (uint8_t*)new_sprite;
    },
    .free_sprite = [](uint8_t* s){
      ((ILI9341Sprite*)s)->free();
    },
    .switch_to_sprite = [](uint8_t* s){
      sprite = (ILI9341Sprite*)s;
    },
    .switch_to_screen = []{
      sprite = screen_sprite;
    },

    .fill_screen = [](uint16_t colour) {
      sprite->fill(colour);
    },
    .draw_char = [](int64_t x, int64_t y, uint8_t c) {
      sprite->cursor_x = x;
      sprite->cursor_y = y;
      sprite->draw_char((char)c);
    },
    .draw_line = [](int64_t x1, int64_t y1, int64_t x2, int64_t y2, uint16_t colour) {
      sprite->draw_line(x1, y1, x2, y2, colour);
    },
    .draw_rect = [](int64_t x, int64_t y, int64_t w, int64_t h, uint16_t colour, bool filled, uint16_t radius) {
      sprite->draw_rect(x, y, w, h, radius, filled, colour);
    },
    .draw_sprite = [](int64_t x, int64_t y, uint8_t *s){
      sprite->draw_sprite(x, y, (ILI9341Sprite*)s);
    },
    .draw_bitmap = [](int64_t x, int64_t y, const uint8_t* bitmap) {
      sprite->draw_bitmap(x, y, get_bitmap_by_name((char*)bitmap));
    },

    .print = [](const uint8_t *s) {
      sprite->draw_string((char*)s);
    },
    .set_cursor = [](int64_t x, int64_t y) {
      sprite->cursor_x = x;
      sprite->cursor_y = y;
    },
    .get_cursor = [](int64_t *x, int64_t *y) {
      *x = sprite->cursor_x;
      *y = sprite->cursor_y;
    },

    .draw = []() {
      tft.draw_sprite(0, 0, screen_sprite);
    },
  },

  .buttons = ButtonsInterface {
    .wait_input_event = [](ButtonInput *input, ButtonEvent *event) {
      return buttons.get_event_input(*input, *event, true);
    },
    .immediate_input_event = [](ButtonInput *input, ButtonEvent *event) {
      return buttons.get_event_input(*input, *event, false);
    },
  },

  .storage = {
    .connected = []() { return storage.connected(); },
    .busy = []() { return storage.busy(); },
    
    .write = [](uint16_t address, uint8_t count, const uint8_t *buffer) {
      return storage.write(address, count, buffer);
    },
    .read = [](uint16_t address, uint8_t count, uint8_t *buffer) {
      return storage.read(address, count, buffer);
    },
  }
};

int main() {
  // Initialize IO and ADC
  stdio_init_all();
  adc_init();

  // Initialize I2C bus
  i2c_init(i2c0, 100000);
  gpio_set_function(I2C_SDA_PIN, GPIO_FUNC_I2C);
  gpio_set_function(I2C_SCL_PIN, GPIO_FUNC_I2C);
  gpio_pull_up(I2C_SDA_PIN);
  gpio_pull_up(I2C_SCL_PIN);

  // Begin peripherals which need beginning
  buttons.begin();
  tft.begin();

  // Set up screen sprite and switch to it
  screen_sprite = tft.create_sprite(TFT_WIDTH, TFT_HEIGHT);
  screen_sprite->fill(0);
  screen_sprite->font = (uint8_t**)droid_sans_20_font;
  screen_sprite->font_colour = 0xFFFF;
  sprite = screen_sprite;

  // Pass the Rust side our HAL struct and let it take over
  delta_pico_set_framework(&framework_interface);
  delta_pico_main();
}
