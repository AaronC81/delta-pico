#include "application.hpp"
#include "application_framework.hpp"
#include "hardware.hpp"
#include <TFT_eSPI.h>

extern "C" {
    #include <delta_pico_rust.h>
}

void rbopRendererClear();
void rbopRendererDrawLine(int64_t x1, int64_t y1, int64_t x2, int64_t y2);
void rbopRendererDrawChar(int64_t x, int64_t y, uint8_t c);

class CalculatorApplication : Application {
public:
    RbopRendererInterface renderer = {
        .clear = rbopRendererClear,
        .draw_char = rbopRendererDrawChar,
        .draw_line = rbopRendererDrawLine,
    };
    RbopContext *ctx;

    void init() override;
    void tick() override;
};
