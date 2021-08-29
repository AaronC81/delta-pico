#include "application_framework.hpp"

extern "C" {
  #include <delta_pico_rust.h>
  #include <DroidSans-20.h>
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

void displayDraw() {
  ApplicationFramework::instance.draw();
}

bool buttonsPollInputEvent(ButtonInput *input, ButtonEvent *event) {
  return ApplicationFramework::instance.buttons().waitForEventInput(*input, *event);
}

void panicHandler(const uint8_t *message) {
  Serial.println("=== PANIC ===");
  Serial.println((char*)message);
  Serial.println("=== PANIC ===");

  ApplicationFramework::instance.sprite().setCursor(0, 0);
  ApplicationFramework::instance.sprite().println("PANIC!");
  ApplicationFramework::instance.sprite().println((const char*)message);
  
  ApplicationFramework::instance.draw();
}

void debugHandler(const uint8_t *message) {
  Serial.println((const char*)message);
}

auto framework_interface = ApplicationFrameworkInterface {
  .panic_handler = panicHandler,
  .debug_handler = debugHandler,
  .display = DisplayInterface {
    .width = IWIDTH,
    .height = IHEIGHT,

    .fill_screen = displayFillScreen,
    .draw_char = displayDrawChar,
    .draw_line = displayDrawLine,
    .draw_rect = displayDrawRect,

    .print = displayPrint,
    .set_cursor = displaySetCursor,

    .draw = displayDraw,
  },
  .buttons = ButtonsInterface {
    .poll_input_event = buttonsPollInputEvent,
  }
};

void setup() {
  // TODO: if serial isn't connected, the entire calculator eventually hangs
  Serial.begin(115200);

  ApplicationFramework::instance.initialize();
  delta_pico_set_framework(&framework_interface);

  ApplicationFramework::instance.sprite().loadFont(DroidSans_20_vlw);

  delta_pico_main();
}

void loop() {}
