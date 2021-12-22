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

PCF8574 col_pcf(i2c0, I2C_EXPANDER_ADDRESS_1);
PCF8574 row_pcf(i2c0, I2C_EXPANDER_ADDRESS_2);
ButtonMatrix buttons(row_pcf, col_pcf);
CAT24C storage(i2c0, CAT24C_ADDRESS);

ApplicationFrameworkInterface framework_interface = ApplicationFrameworkInterface {
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
      tud_task();
      return buttons.get_event_input(*input, *event, true);
    },
    .immediate_input_event = [](ButtonInput *input, ButtonEvent *event) {
      tud_task();
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
  },

  .usb_mass_storage = {
    .fat12_filesystem = NULL,
    .active = false,
    .enter = []() {
      framework_interface.usb_mass_storage.active = true;
      usb_mass_storage_ejected = false;

      tusb_init();
      while (framework_interface.usb_mass_storage.active && !usb_mass_storage_ejected) {
        tud_task();
      }
      tud_disconnect();

      return true;
    }
  }
};


// TODO: Temporary: assign USB mass storage FAT12 filesystem to a static image
// In the future, this will be read off the EEPROM by the Rust side instead

#define README_CONTENTS \
    "Your Delta Pico is mounted as USB flash storage. Add, edit, or remove files, " \
    "then eject the drive in your operating system."

uint8_t default_fat12_fs[USB_MASS_STORAGE_BLOCK_NUM][USB_MASS_STORAGE_BLOCK_SIZE] =
{
    //------------- Block0: Boot Sector -------------//
    // byte_per_sector    = USB_MASS_STORAGE_BLOCK_SIZE; fat12_sector_num_16  = USB_MASS_STORAGE_BLOCK_NUM;
    // sector_per_cluster = 1; reserved_sectors = 1;
    // fat_num            = 1; fat12_root_entry_num = 16;
    // sector_per_fat     = 1; sector_per_track = 1; head_num = 1; hidden_sectors = 0;
    // drive_number       = 0x80; media_type = 0xf8; extended_boot_signature = 0x29;
    // filesystem_type    = "FAT12   "; volume_serial_number = 0x1234; volume_label = "TinyUSB 0  ";
    // FAT magic code at offset 510-511
    {
        0xEB, 0x3C, 0x90, 0x4D, 0x53, 0x44, 0x4F, 0x53, 0x35, 0x2E, 0x30, 0x00, 0x02, 0x01, 0x01, 0x00,
        0x01, 0x10, 0x00, 0x10, 0x00, 0xF8, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x29, 0x34, 0x12, 0x00, 0x00, 'D' , 'e' , 'l' , 't' , 'a' ,
        ' ' , 'P' , 'i' , 'c' , 'o' , ' ' , 0x46, 0x41, 0x54, 0x31, 0x32, 0x20, 0x20, 0x20, 0x00, 0x00,

        // Zero up to 2 last bytes of FAT magic code
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x55, 0xAA
    },

    //------------- Block1: FAT12 Table -------------//
    {
        0xF8, 0xFF, 0xFF, 0xFF, 0x0F // // first 2 entries must be F8FF, third entry is cluster end of readme file
    },

    //------------- Block2: Root Directory -------------//
    {
        // first entry is volume label
        'D' , 'e' , 'l' , 't' , 'a' , ' ' , 'P' , 'i' , 'c' , 'o' , ' ' , 0x08, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4F, 0x6D, 0x65, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // second entry is readme file
        'R' , 'E' , 'A' , 'D' , 'M' , 'E' , ' ' , ' ' , 'T' , 'X' , 'T' , 0x20, 0x00, 0xC6, 0x52, 0x6D,
        0x65, 0x43, 0x65, 0x43, 0x00, 0x00, 0x88, 0x6D, 0x65, 0x43, 0x02, 0x00,
        sizeof(README_CONTENTS)-1, 0x00, 0x00, 0x00 // readme's files size (4 Bytes)
    },

    //------------- Block3: Readme Content -------------//
    README_CONTENTS
};

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

  // Begin peripherals which need beginning
  buttons.begin();
  tft.begin();

  // Set up screen sprite and switch to it
  screen_sprite = tft.create_sprite(TFT_WIDTH, TFT_HEIGHT);
  screen_sprite->fill(0);
  screen_sprite->font = (uint8_t**)droid_sans_20_font;
  screen_sprite->font_colour = 0xFFFF;
  sprite = screen_sprite;

  // TODO: Temporary
  usb_mass_storage_fat12_filesystem = &default_fat12_fs;

  // Pass the Rust side our HAL struct and let it take over
  delta_pico_set_framework(&framework_interface);
  delta_pico_main();
}


// Invoked when device is mounted
void tud_mount_cb(void)
{
}

// Invoked when device is unmounted
void tud_umount_cb(void)
{
}

// Invoked when usb bus is suspended
// remote_wakeup_en : if host allow us  to perform remote wakeup
// Within 7ms, device must draw an average of current less than 2.5 mA from bus
void tud_suspend_cb(bool remote_wakeup_en)
{
}

// Invoked when usb bus is resumed
void tud_resume_cb(void)
{
}

//--------------------------------------------------------------------+
// BLINKING TASK
//--------------------------------------------------------------------+
void led_blinking_task(void)
{
}
