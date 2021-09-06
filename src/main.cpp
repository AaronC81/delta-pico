#include "application_framework.hpp"

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

  ApplicationFramework::instance.initialize();
  delta_pico_set_framework(&framework_interface);

  delta_pico_main();
}

void loop() {}
