#pragma once

#include <stdint.h>

enum token {
    TOKEN_NONE = 0,
    TOKEN_0 = 1,
    TOKEN_1 = 2,
    TOKEN_2 = 3,
    TOKEN_3 = 4,
    TOKEN_4 = 5,
    TOKEN_5 = 6,
    TOKEN_6 = 7,
    TOKEN_7 = 8,
    TOKEN_8 = 9,
    TOKEN_9 = 10,
    TOKEN_LPAREN = 11,
    TOKEN_RPAREN = 12,
    TOKEN_PLUS = 13,
    TOKEN_SUBTRACT = 14,
    TOKEN_MULTIPLY = 15,
    TOKEN_DIVIDE = 16,
    TOKEN_NEGATE = 17,
} __attribute__((__packed__));

#define TOKEN_LIMIT 128

// Type large enough to index into array of size TOKEN_LIMIT + 1
#define token_index_t uint8_t

// Make sure the token enum is nice and small
static_assert(sizeof(token) == 1, "token enum is too large");

#define TOKEN_BITMAPS_LENGTH 18

extern const uint8_t *token_bitmaps[TOKEN_BITMAPS_LENGTH];

bool token_is_binop(token t);
uint8_t token_operator_precedence(token t);
