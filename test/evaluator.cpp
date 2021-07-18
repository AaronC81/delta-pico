#include <assert.h>
#include <string.h>
#include <cstdio>

#include "maths/evaluator.hpp"
#include "maths/tokens.hpp"

#define TOKENS(...) ((const enum token[]){ __VA_ARGS__ })
#define TOKENS_L(...) TOKENS(__VA_ARGS__), (sizeof(TOKENS(__VA_ARGS__)) / sizeof(enum token))

#define PF_OP(o) ((struct evaluator_postfix_item){ \
        .is_operator = true, .value = { .op = o } \
    })
#define PF_NUM(num) ((struct evaluator_postfix_item){ \
        .is_operator = false, .value = { .number = num } \
    })
#define PF_ITEMS(...) ((const struct evaluator_postfix_item[]){ __VA_ARGS__ })
#define PF_ITEMS_L(...) PF_ITEMS(__VA_ARGS__), (sizeof(PF_ITEMS(__VA_ARGS__)) / sizeof(struct evaluator_postfix_item))



void test_expression(const enum token* tokens, token_index_t tokens_length, evaluator_t expected_result) {
    // Shunt
    struct evaluator_postfix_item items[TOKEN_LIMIT];
    token_index_t items_length;
    assert(evaluator_shunt((enum token*)tokens, tokens_length, items, &items_length) == EVALUATOR_STATUS_OK);

    // Evaluate
    evaluator_t result;
    assert(evaluator_evaluate(items, items_length, &result) == EVALUATOR_STATUS_OK);
    assert(result == expected_result);
}

void test_shunt(
    const enum token* tokens, token_index_t tokens_length,
    const struct evaluator_postfix_item* expected_items, token_index_t expected_items_length
) {
    struct evaluator_postfix_item actual_items[TOKEN_LIMIT];
    token_index_t actual_items_length;

    assert(evaluator_shunt((enum token*)tokens, tokens_length, actual_items, &actual_items_length) == EVALUATOR_STATUS_OK);

    assert(actual_items_length == expected_items_length);

    for (token_index_t i = 0; i < actual_items_length; i++) {
        assert(actual_items[i].is_operator == expected_items[i].is_operator);

        if (actual_items[i].is_operator) {
            assert(actual_items[i].value.op == expected_items[i].value.op);
        } else {
            assert(actual_items[i].value.number == expected_items[i].value.number);
        }
    }
}

int main(void) {
    // Test shunting a digit
    test_shunt(
        TOKENS_L(TOKEN_6),
        PF_ITEMS_L(PF_NUM(6))
    );


    // Test shunting a simple addition
    test_shunt(
        TOKENS_L(TOKEN_6, TOKEN_PLUS, TOKEN_2),
        PF_ITEMS_L(PF_NUM(6), PF_NUM(2), PF_OP(TOKEN_PLUS))
    );


    // Test shunting an expression with relevant precedence
    test_shunt(
        TOKENS_L(TOKEN_6, TOKEN_PLUS, TOKEN_2, TOKEN_MULTIPLY, TOKEN_3, TOKEN_PLUS, TOKEN_7),
        PF_ITEMS_L(PF_NUM(6), PF_NUM(2), PF_NUM(3), PF_OP(TOKEN_MULTIPLY), PF_OP(TOKEN_PLUS), PF_NUM(7), PF_OP(TOKEN_PLUS))
    );


    // Test shunting a negative number
    test_shunt(
        TOKENS_L(TOKEN_SUBTRACT, TOKEN_3),
        PF_ITEMS_L(PF_NUM(3), PF_OP(TOKEN_NEGATE))
    );


    // Test shunting an expression which uses unary operators
    test_shunt(
        // -3+-(-6-+2)
        TOKENS_L(TOKEN_SUBTRACT, TOKEN_3, TOKEN_PLUS, TOKEN_SUBTRACT, TOKEN_LPAREN, TOKEN_SUBTRACT, TOKEN_6, TOKEN_SUBTRACT, TOKEN_PLUS, TOKEN_2, TOKEN_RPAREN),
        PF_ITEMS_L(PF_NUM(3), PF_OP(TOKEN_NEGATE), PF_NUM(6), PF_OP(TOKEN_NEGATE), PF_NUM(2), PF_OP(TOKEN_SUBTRACT), PF_OP(TOKEN_NEGATE), PF_OP(TOKEN_PLUS))
    );


    // Test shunting consecutive unary ops
    test_shunt(
        TOKENS_L(TOKEN_SUBTRACT, TOKEN_SUBTRACT, TOKEN_3),
        PF_ITEMS_L(PF_NUM(3))
    );


    // Test digit accept
    test_expression(
        TOKENS_L(TOKEN_6),
        6
    );


    // Test integer evaluation
    test_expression(
        TOKENS_L(TOKEN_1, TOKEN_0, TOKEN_2),
        102
    );


    // Test expression evaluation
    test_expression(
        TOKENS_L(TOKEN_1, TOKEN_PLUS, TOKEN_2, TOKEN_MULTIPLY, TOKEN_2),
        5
    );

    test_expression(
        TOKENS_L(TOKEN_LPAREN, TOKEN_1, TOKEN_PLUS, TOKEN_2, TOKEN_RPAREN, TOKEN_MULTIPLY, TOKEN_2),
        6
    );

    test_expression(
        TOKENS_L(TOKEN_LPAREN, TOKEN_1, TOKEN_PLUS, TOKEN_2, TOKEN_RPAREN, TOKEN_MULTIPLY, TOKEN_2),
        6
    );

    test_expression(
        TOKENS_L(TOKEN_5, TOKEN_SUBTRACT, TOKEN_2, TOKEN_SUBTRACT, TOKEN_4),
        -1
    );

    test_expression(
        TOKENS_L(TOKEN_SUBTRACT, TOKEN_3),
        -3
    );
    
    test_expression(
        TOKENS_L(TOKEN_SUBTRACT, TOKEN_3, TOKEN_PLUS, TOKEN_SUBTRACT, TOKEN_6),
        -9
    );
}
