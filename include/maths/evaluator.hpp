#pragma once

#include "maths/tokens.hpp"

// TODO: bad bad bad bad
//       In a *calculator* of all things, we don't want binary inaccuracies
//       Implement a better format later on!
#define evaluator_t double

enum evaluator_status {
    EVALUATOR_STATUS_OK = 0,
    EVALUATOR_STATUS_SYNTAX_ERROR,
};

struct evaluator_postfix_item {
    bool is_operator;
    union {
        evaluator_t number;
        token op;
    } value;
};

evaluator_status evaluator_shunt(
    token *tokens, token_index_t tokens_length,
    struct evaluator_postfix_item *output, token_index_t *output_length
);

evaluator_status evaluator_evaluate(
    struct evaluator_postfix_item *items, token_index_t items_length,
    evaluator_t *result
);
