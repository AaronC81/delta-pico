#include "application_framework.hpp"
#include "bitmap.h"

#include "mbed.h"

extern "C" {
  #include <delta_pico_rust.h>
}

void displayFillScreen(uint16_t colour) {
    ApplicationFramework::instance.sprite().fillScreen(colour);
}

void displayDrawLine(int64_t x1, int64_t y1, int64_t x2, int64_t y2, uint16_t colour) {
    ApplicationFramework::instance.sprite().drawLine(x1, y1, x2, y2, colour);
}

void displayDrawChar(int64_t x, int64_t y, uint8_t c) {
    ApplicationFramework::instance.sprite().setCursor(x, y);
    ApplicationFramework::instance.sprite().print((char)c);
}

void displayDrawRect(int64_t x, int64_t y, int64_t w, int64_t h, uint16_t colour, bool fill, uint16_t radius) {
  if (fill) {
    ApplicationFramework::instance.sprite().fillRoundRect(x, y, w, h, radius, colour);
  } else {
    ApplicationFramework::instance.sprite().drawRoundRect(x, y, w, h, radius, colour);
  }
}

void displayPrint(const uint8_t *s) {
  ApplicationFramework::instance.sprite().print((char*)s);
}

void displaySetCursor(int64_t x, int64_t y) {
  ApplicationFramework::instance.sprite().setCursor(x, y);
}

void displayGetCursor(int64_t *x, int64_t *y) {
  *x = ApplicationFramework::instance.sprite().getCursorX();
  *y = ApplicationFramework::instance.sprite().getCursorY();
}

void displayDrawBitmap(int64_t sx, int64_t sy, uint16_t *bitmap) {
  int width = bitmap[0];
  int height = bitmap[1];
  int transparency = bitmap[2];

  for (int x = 0; x < width; x++) {
    for (int y = 0; y < height; y++) {
      int index = x * height + y + 3;

      if (bitmap[index] != transparency) {
        ApplicationFramework::instance.sprite().drawPixel(sx + x, sy + y, bitmap[index]);
      }
    }
  }
}

void displayDraw() {
  ApplicationFramework::instance.draw();
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

void panicHandler(const uint8_t *message) {
  // if (Serial.available()) {
  //   Serial.println("=== PANIC ===");
  //   Serial.println((char*)message);
  //   Serial.println("=== PANIC ===");
  // }

  ApplicationFramework::instance.sprite().setCursor(0, 0);
  ApplicationFramework::instance.sprite().println("PANIC!");

  // Chunk the string into 20-character lines
  String result;
  uint32_t idx = 0;
  while (message[idx] != 0) {
    result.concat((char)message[idx]);
    idx++;
    if (idx % 20 == 0) {
      result.concat("\n");
    }
  }

  ApplicationFramework::instance.sprite().println(result.c_str());
  
  ApplicationFramework::instance.draw();
}

void debugHandler(const uint8_t *message) {
  // if (Serial.available()) {
  //   Serial.println((const char*)message);
  // }
}

auto framework_interface = ApplicationFrameworkInterface {
  .panic_handler = panicHandler,
  .debug_handler = debugHandler,
  .millis = millis,
  .micros = micros,
  .charge_status = []() -> int32_t {
    int adcReading = analogRead(A3);

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
    mbed_stats_heap_t heap_stats;
    mbed_stats_heap_get(&heap_stats);

    *used = heap_stats.current_size;
    *available = heap_stats.reserved_size;
  },

  .display = DisplayInterface {
    .width = IWIDTH,
    .height = IHEIGHT,

    .new_sprite = [](int16_t w, int16_t h){ return (uint8_t*)ApplicationFramework::instance.newSprite(w, h); },
    .free_sprite = [](uint8_t* s){ ApplicationFramework::instance.freeSprite((TFT_eSprite*)s); },
    .switch_to_sprite = [](uint8_t* s){ ApplicationFramework::instance.switchToSprite((TFT_eSprite*)s); },
    .switch_to_screen = []{ ApplicationFramework::instance.switchToScreen(); },

    .fill_screen = displayFillScreen,
    .draw_char = displayDrawChar,
    .draw_line = displayDrawLine,
    .draw_rect = displayDrawRect,
    .draw_sprite = [](int64_t x, int64_t y, uint8_t *s){
      auto sprite = (TFT_eSprite*)s;
      ApplicationFramework::instance.sprite().pushImage(
        x, y, sprite->width(), sprite->height(), (uint16_t*)sprite->getPointer(), SOFTWARE_COLOR_DEPTH
      );
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

void setup() {
  // TODO: if serial isn't connected, the entire calculator eventually hangs
  Serial.begin(115200);

  pinMode(A3, INPUT);

  ApplicationFramework::instance.initialize();
  delta_pico_set_framework(&framework_interface);

  delta_pico_main();
}

void loop() {}
