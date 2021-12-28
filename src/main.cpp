#include "bitmap.h"

#include "pico/time.h"
#include "pico/stdlib.h"
#include "hardware/gpio.h"
#include "hardware/adc.h"
#include "hardware/irq.h"
#include "pico/multicore.h"
#include "pico/util/queue.h"

extern "C" {
  #include "hardware.h"
}

#include "pcf8574.hpp"
#include "button_matrix.hpp"
#include "cat24c.hpp"
#include "ili9341.hpp"
#include "usb_mass_storage.h"

#include <stdio.h>

#include "tusb_config.h"
#include "tusb.h"

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

CAT24C storage(i2c0, CAT24C_ADDRESS);

const char *hardware_revision = DELTA_PICO_REVISION_NAME;

typedef struct {
  ButtonInput input;
  ButtonEvent event;
} ButtonInputEvent;
const size_t BUTTON_QUEUE_SIZE = 32;
queue_t button_queue;
volatile bool button_queue_enabled = true;


static void usb_interrupt_worker_irq(void) {
  tud_task();
}

static int64_t usb_interrupt_timer_task(__unused alarm_id_t id, __unused void *user_data) {
  irq_set_pending(31);
  return USB_INTERRUPT_INTERVAL_US;
}

ApplicationFrameworkInterface framework_interface = ApplicationFrameworkInterface {
  .debug_handler = [](const uint8_t *string) {
    if (tusb_inited() && tud_cdc_connected()) {
      tud_cdc_write_str((const char*)string);
      tud_cdc_write_char('\r');
      tud_cdc_write_char('\n');
      tud_cdc_write_flush();
      
      tud_task();
    }
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

    #ifdef DELTA_PICO_TRAIT_BATTERY_VOLTAGE_DROP
      voltage += DELTA_PICO_TRAIT_BATTERY_VOLTAGE_DROP;
    #endif

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

  .hardware_revision = (uint8_t*)hardware_revision,

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
      ButtonInputEvent input_event;
      queue_remove_blocking(&button_queue, &input_event);

      *input = input_event.input;
      *event = input_event.event;

      return true;
    },
    .immediate_input_event = [](ButtonInput *input, ButtonEvent *event) {      
      ButtonInputEvent input_event;
      if (queue_try_remove(&button_queue, &input_event)) {
        *input = input_event.input;
        *event = input_event.event;

        return true;
      } else {
        return false;
      }
    },
  },

  .storage = {
    .connected = []() { return storage.connected(); },
    .busy = []() { return storage.busy(); },
    
    .write = [](uint16_t address, uint16_t count, const uint8_t *buffer) {
      return storage.write(address, count, buffer);
    },
    .read = [](uint16_t address, uint16_t count, uint8_t *buffer) {
      return storage.read(address, count, buffer);
    },

    .acquire_priority = []() { button_queue_enabled = false; },
    .release_priority = []() { button_queue_enabled = true; }
  },

  .usb_mass_storage = {
    .block_num = USB_MASS_STORAGE_BLOCK_NUM,
    .block_size = USB_MASS_STORAGE_BLOCK_SIZE,
    .fat12_filesystem = NULL,

    .active = false,
    .begin = []() {
      tusb_init();

      // Set up periodic handler to deal with USB stuff
      irq_set_exclusive_handler(USB_INTERRUPT_IRQ, usb_interrupt_worker_irq);
      irq_set_enabled(USB_INTERRUPT_IRQ, true);
      add_alarm_in_us(USB_INTERRUPT_INTERVAL_US, usb_interrupt_timer_task, NULL, true);

      usb_mass_storage_fat12_filesystem = framework_interface.usb_mass_storage.fat12_filesystem;

      return true;
    }
  }
};

void core1_main() {
  // Initialise button matrix
  PCF8574 col_pcf(i2c0, I2C_EXPANDER_ADDRESS_1);
  PCF8574 row_pcf(i2c0, I2C_EXPANDER_ADDRESS_2);
  ButtonMatrix buttons(row_pcf, col_pcf);
  buttons.begin();

  while (1) {
    ButtonInput input;
    ButtonEvent event;

    if (button_queue_enabled) {
      if (buttons.get_event_input(input, event, false)) {
        ButtonInputEvent input_event = { .input = input, .event = event };
        queue_add_blocking(&button_queue, &input_event);
      }
    }
  }
}

int main() {
  // Initialize IO and ADC
  // stdio_init_all();
  adc_init();

  // Initialize I2C bus
  i2c_init(i2c0, 100000);
  gpio_set_function(I2C_SDA_PIN, GPIO_FUNC_I2C);
  gpio_set_function(I2C_SCL_PIN, GPIO_FUNC_I2C);
  gpio_pull_up(I2C_SDA_PIN);
  gpio_pull_up(I2C_SCL_PIN);
  recursive_mutex_init(&i2c_mutex);

  // Begin peripherals which need beginning
  tft.begin();

  // Set up screen sprite and switch to it
  screen_sprite = tft.create_sprite(TFT_WIDTH, TFT_HEIGHT);
  screen_sprite->fill(0);
  screen_sprite->font = (uint8_t**)droid_sans_20_font;
  screen_sprite->font_colour = 0xFFFF;
  sprite = screen_sprite;

  // Set up button queue and kick off core 1
  queue_init(&button_queue, sizeof(ButtonInputEvent), BUTTON_QUEUE_SIZE);
  multicore_launch_core1(core1_main);

  // Pass the Rust side our HAL struct and let it take over
  delta_pico_set_framework(&framework_interface);
  delta_pico_main();
}
