#include "applications/calculator.hpp"

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

#define I RbopInput

#ifdef DELTA_PICO_PROTOTYPE
const RbopInput buttonMapping[7][7] = {
  { I::None,      I::MoveUp,    I::None,      I::None,      I::None,      I::None,      I::None, },
  { I::MoveLeft,  I::None,      I::MoveRight, I::None,      I::None,      I::None,      I::None, },
  { I::None,      I::MoveDown,  I::None,      I::None,      I::None,      I::None,      I::None, },
  { I::Digit7,    I::Digit8,    I::Digit9,    I::Delete,    I::None,      I::None,      I::None, },
  { I::Digit4,    I::Digit5,    I::Digit6,    I::Multiply,  I::Fraction,  I::None,      I::None, },
  { I::Digit1,    I::Digit2,    I::Digit3,    I::Add,       I::Subtract,  I::None,      I::None, },
  { I::Digit0,    I::Point,     I::None,      I::None,      I::None,      I::None,      I::None, },
};
#endif

#ifdef DELTA_PICO_REV1
const RbopInput buttonMapping[7][7] = {
  { I::MoveUp, I::MoveRight, I::None, I::None, I::None, I::None, I::None, },
  { I::MoveLeft, I::MoveDown, I::None, I::None, I::None, I::None, I::None, },
  { I::Digit7, I::Digit8, I::Digit9, I::Delete, I::None, I::None, I::None, },
  { I::Digit4, I::Digit5, I::Digit6, I::Multiply, I::None, I::None, I::Fraction, },
  { I::None, I::None, I::None, I::None, I::None, I::None, I::None, },
  { I::Digit0, I::None, I::None, I::None, I::None, I::None, I::None, },
  { I::Digit1, I::Digit2, I::Digit3, I::Add, I::None, I::None, I::Subtract, },
};
#endif

#undef I

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

    uint8_t r, c;
    ButtonEvent evt;
    if (ApplicationFramework::instance.buttons().waitForEvent(r, c, evt) && evt == ButtonEvent::PRESS) {
        RbopInput input;

        rbop_input(ctx, buttonMapping[r][c]);
    }
}
