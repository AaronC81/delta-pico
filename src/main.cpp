#include "application_framework.hpp"

extern "C" {
  #include <delta_pico_rust.h>
  #include <DroidSansMono-20.h>
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

void displayPrint(const uint8_t *s) {
  ApplicationFramework::instance.sprite().print((char*)s);
}

void displaySetCursor(int64_t x, int64_t y) {
  ApplicationFramework::instance.sprite().setCursor(x, y);
}

void displayDraw() {
  ApplicationFramework::instance.draw();
}

bool buttonsPollInputEvent(RbopInput *input, ButtonEvent *event) {
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
    .fill_screen = displayFillScreen,
    .draw_char = displayDrawChar,
    .draw_line = displayDrawLine,
    .print = displayPrint,
    .set_cursor = displaySetCursor,
    .draw = displayDraw,
  },
  .buttons = ButtonsInterface {
    .poll_input_event = buttonsPollInputEvent,
  }
};

void setup() {
  Serial.begin(115200);

  ApplicationFramework::instance.initialize();
  rbop_set_framework(&framework_interface);

  ApplicationFramework::instance.sprite().loadFont(DroidSansMono_20_vlw);

  delta_pico_main();
}

void loop() {}
