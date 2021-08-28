#include "application_framework.hpp"
#include "applications/calculator.hpp"

extern "C" {
  #include <delta_pico_rust.h>
  #include <DroidSansMono-20.h>
}

void rbopPanicHandler(const uint8_t *message) {
  ApplicationFramework::instance.sprite().setCursor(0, 0);
  ApplicationFramework::instance.sprite().println("PANIC!");
  ApplicationFramework::instance.sprite().println((const char*)message);
  
  ApplicationFramework::instance.draw();
}

void rbopDebugHandler(const uint8_t *message) {
  Serial.println((const char*)message);
}

CalculatorApplication calcApp;

void setup() {
  rbop_set_panic_handler(rbopPanicHandler);

  Serial.begin(115200);
  rbop_set_debug_handler(rbopDebugHandler);

  ApplicationFramework::instance.initialize();
  ApplicationFramework::instance.sprite().loadFont(DroidSansMono_20_vlw);

  calcApp.init();
}

void loop() {
  calcApp.tick();
}
