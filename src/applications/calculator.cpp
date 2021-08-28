#include "applications/calculator.hpp"
#include "hardware.hpp"

void rbopRendererClear() {
    ApplicationFramework::instance.sprite().fillScreen(TFT_BLACK);
}

void rbopRendererDrawLine(int64_t x1, int64_t y1, int64_t x2, int64_t y2) {
    ApplicationFramework::instance.sprite().drawLine(x1, y1, x2, y2, TFT_WHITE);
}

void rbopRendererDrawChar(int64_t x, int64_t y, uint8_t c) {
    ApplicationFramework::instance.sprite().setCursor(x, y);
    ApplicationFramework::instance.sprite().print((char)c);
}

void CalculatorApplication::init() {
    ctx = rbop_new(&renderer);
    rbop_set_viewport(ctx, SWIDTH, SHEIGHT);
}

void CalculatorApplication::tick() {
    rbop_render(ctx);

    double result;
    if (rbop_evaluate(ctx, &result)) {
        ApplicationFramework::instance.sprite().setCursor(0, SHEIGHT - 30);
        ApplicationFramework::instance.sprite().print(result);
    }

    ApplicationFramework::instance.draw();

    RbopInput input;
    ButtonEvent evt;
    if (ApplicationFramework::instance.buttons().waitForEventInput(input, evt) && evt == ButtonEvent::PRESS) {
        rbop_input(ctx, input);
    }
}
