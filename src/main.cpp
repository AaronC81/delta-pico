#include "application_framework.hpp"
#include "bitmap.h"

#include "pico/time.h"

extern "C" {
  #include <delta_pico_rust.h>
}

void displayFillScreen(uint16_t colour) {
    //ApplicationFramework::instance.sprite().fillScreen(colour);
}

void displayDrawLine(int64_t x1, int64_t y1, int64_t x2, int64_t y2, uint16_t colour) {
    //ApplicationFramework::instance.sprite().drawLine(x1, y1, x2, y2, colour);
}

void displayDrawChar(int64_t x, int64_t y, uint8_t c) {
    // ApplicationFramework::instance.sprite().setCursor(x, y);
    // ApplicationFramework::instance.sprite().print((char)c);
}

void displayDrawRect(int64_t x, int64_t y, int64_t w, int64_t h, uint16_t colour, bool fill, uint16_t radius) {
  // if (fill) {
  //   ApplicationFramework::instance.sprite().fillRoundRect(x, y, w, h, radius, colour);
  // } else {
  //   ApplicationFramework::instance.sprite().drawRoundRect(x, y, w, h, radius, colour);
  // }
}

void displayPrint(const uint8_t *s) {
  // ApplicationFramework::instance.sprite().print((char*)s);
}

void displaySetCursor(int64_t x, int64_t y) {
  // ApplicationFramework::instance.sprite().setCursor(x, y);
}

void displayGetCursor(int64_t *x, int64_t *y) {
  // *x = ApplicationFramework::instance.sprite().getCursorX();
  // *y = ApplicationFramework::instance.sprite().getCursorY();
}

void displayDrawBitmap(int64_t sx, int64_t sy, uint16_t *bitmap) {
  // if (bitmap == nullptr) return;

  // uint16_t width = bitmap[0];
  // uint16_t height = bitmap[1];
  // uint16_t transparency = bitmap[2];
  // uint16_t runLength = bitmap[3];

  // int index = 4;
  // for (uint16_t x = 0; x < width; x++) {
  //   for (uint16_t y = 0; y < height; y++) {
  //     if (bitmap[index] == runLength) {
  //       uint16_t times = bitmap[index + 1];
  //       uint16_t colour = bitmap[index + 2];

  //       if (colour != transparency) {
  //         for (uint16_t i = 0; i < times; i++) {
  //           ApplicationFramework::instance.sprite().drawPixel(sx + x, sy + y + i, colour);
  //         }
  //       }

  //       y += times - 1;
  //       index += 3;
  //     } else {
  //       if (bitmap[index] != transparency) {
  //         ApplicationFramework::instance.sprite().drawPixel(sx + x, sy + y, bitmap[index]);
  //       }
  //       index++;
  //     }
  //   }
  // }
}

void displayDraw() {
  // ApplicationFramework::instance.draw();
}

bool buttonsWaitInputEvent(ButtonInput *input, ButtonEvent *event) {
  return ApplicationFramework::instance.buttons().getEventInput(*input, *event, true);
}

bool buttonsImmediateInputEvent(ButtonInput *input, ButtonEvent *event) {
  return ApplicationFramework::instance.buttons().getEventInput(*input, *event, false);
}

bool storageConnected() {
  return ApplicationFramework::instance.storage().connected();
}

bool storageBusy() {
  return ApplicationFramework::instance.storage().busy();
}

bool storageWrite(uint16_t address, uint8_t count, const uint8_t *buffer) {
  return ApplicationFramework::instance.storage().write(address, count, buffer);
}

bool storageRead(uint16_t address, uint8_t count, uint8_t *buffer) {
  return ApplicationFramework::instance.storage().read(address, count, buffer);
}

void debugHandler(const uint8_t *message) {
  // if (Serial.available()) {
  //   Serial.println((const char*)message);
  // }
}

auto framework_interface = ApplicationFrameworkInterface {
  .debug_handler = debugHandler,
  .millis = []() -> uint32_t { return to_ms_since_boot(get_absolute_time()); },
  .micros = []() -> uint32_t { return to_us_since_boot(get_absolute_time()); },
  .charge_status = []() -> int32_t {
    // TODO
    // int adcReading = analogRead(A3);
    int adcReading = 50;

    // Divide by resolution, times by Pico logical voltage, times by 3
    // (Voltage is divided by 3 - see Pico Datasheet section 4.4) 
    float voltage = ((float)adcReading / 1024.0) * 3.3 * 3;

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

    .new_sprite = [](int16_t w, int16_t h){ return (uint8_t*)ApplicationFramework::instance.newSprite(w, h); },
    .free_sprite = [](uint8_t* s){
      // TODO
      // ApplicationFramework::instance.freeSprite((TFT_eSprite*)s);
    },
    .switch_to_sprite = [](uint8_t* s){
      // TODO
      // ApplicationFramework::instance.switchToSprite((TFT_eSprite*)s);
    },
    .switch_to_screen = []{ ApplicationFramework::instance.switchToScreen(); },

    .fill_screen = displayFillScreen,
    .draw_char = displayDrawChar,
    .draw_line = displayDrawLine,
    .draw_rect = displayDrawRect,
    .draw_sprite = [](int64_t x, int64_t y, uint8_t *s){
      // auto sprite = (TFT_eSprite*)s;
      // ApplicationFramework::instance.sprite().pushImage(
      //   x, y, sprite->width(), sprite->height(), (uint16_t*)sprite->getPointer(), SOFTWARE_COLOR_DEPTH
      // );
    },
    .draw_bitmap = [](int64_t x, int64_t y, const uint8_t* bitmap) {
      displayDrawBitmap(x, y, getBitmapByName((char*)bitmap));
    },

    .print = displayPrint,
    .set_cursor = displaySetCursor,
    .get_cursor = displayGetCursor,

    .draw = displayDraw,
  },
  .buttons = ButtonsInterface {
    .wait_input_event = buttonsWaitInputEvent,
    .immediate_input_event = buttonsImmediateInputEvent,
  },
  .storage = {
    .connected = storageConnected,
    .busy = storageBusy,
    .write = storageWrite,
    .read = storageRead,
  }
};

int main() {
  // TODO
  // pinMode(A3, INPUT);

  ApplicationFramework::instance.initialize();
  // delta_pico_set_framework(&framework_interface);

  // ILI9341 tft(
  //     spi0,
  //     0, // MISO
  //     3, // MOSI
  //     2, // SCLK
  //     5, // DC
  //     4, // CS
  //     6, // RST
  //     28 // Power
  // );
  // tft.begin();

  // Draw test rectangle
  uint16_t x = 20;
  uint16_t y = 20;
  uint16_t x2 = 39;
  uint16_t y2 = 39;
  uint16_t c = 0xAB;

  ApplicationFramework::instance.tft().writeCommand(0x2A); // CASET
  ApplicationFramework::instance.tft().writeData((x & 0xFF00) >> 8);
  ApplicationFramework::instance.tft().writeData(x & 0x00FF);
  ApplicationFramework::instance.tft().writeData((x2 & 0xFF00) >> 8);
  ApplicationFramework::instance.tft().writeData(x2 & 0x00FF);

  ApplicationFramework::instance.tft().writeCommand(0x2B); // PASET
  ApplicationFramework::instance.tft().writeData((y & 0xFF00) >> 8);
  ApplicationFramework::instance.tft().writeData(y & 0x00FF);
  ApplicationFramework::instance.tft().writeData((y2 & 0xFF00) >> 8);
  ApplicationFramework::instance.tft().writeData(y2 & 0x00FF);

  ApplicationFramework::instance.tft().writeCommand(0x2C); // RAMRW
  for (int i = 0; i < 20 * 20 * 2; i++) {
      ApplicationFramework::instance.tft().writeData(c);
  }

  delta_pico_main();
}
