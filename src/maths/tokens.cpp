#include "maths/tokens.hpp"

#include <stddef.h>

// const uint8_t *token_bitmaps[TOKEN_BITMAPS_LENGTH] = {
//     graphics_token_none,
//     graphics_token_0,
//     graphics_token_1,
//     graphics_token_2,
//     graphics_token_3,
//     graphics_token_4,
//     graphics_token_5,
//     graphics_token_6,
//     graphics_token_7,
//     graphics_token_8,
//     graphics_token_9,
//     graphics_token_lparen,
//     graphics_token_rparen,
//     graphics_token_plus,
//     graphics_token_subtract,
//     graphics_token_multiply,
//     graphics_token_divide,
//     graphics_token_subtract, // Negate - should never be displayed
// };

bool token_is_binop(token t) {
    return
        t == TOKEN_PLUS ||
        t == TOKEN_SUBTRACT ||
        t == TOKEN_MULTIPLY ||
        t == TOKEN_DIVIDE;
}

uint8_t token_operator_precedence(token t) {
    switch (t) {
    case TOKEN_MULTIPLY:
    case TOKEN_DIVIDE:
        return 2;
    case TOKEN_PLUS:
    case TOKEN_SUBTRACT:
        return 1;
    default:
        return 0;
    }
}
